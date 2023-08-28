mod state;
mod terminal_manager;

use crate::command::{
    Blocking, CommandBuilder, ExecutionResult, Interruptible, WasWoken, WithEnv, WithOutput,
};
use crate::config::KeyEvent;
use crate::config::{Config, Keybindings};
use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use terminal_manager::TerminalManager;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub use state::State;
pub use state::{EnvVariable, EnvVariables};

pub type WatchedCommand = CommandBuilder<Blocking, WithEnv, WithOutput, Interruptible>;

pub struct UI {
    blocking_state: BlockingState,
    terminal_manager: TerminalManager,
    state: State,
    watch_rate: Duration,
    keybindings: Arc<Keybindings>,
    remaining_operations: Option<RemainingOperations>,
    channels: Channels,
}

/// After having blocked, there might be some remaining operations, that
/// were originally requested, which we still have to execute.
#[derive(Debug)]
struct RemainingOperations {
    /// The key that is mapped to the remaining operations. Saving this is
    /// more (memory) efficient than copying the an partial Operations type.
    key: KeyEvent,
    /// The index in the Operations vector where the remaining operations start.
    remaining_index: usize,
}

/// All mpsc channels we save in the UI.
struct Channels {
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
    /// We don't store the receivers for the reload and subcommand channels,
    /// because their ownership is passed to the polling tasks.
    reload_tx: Sender<InterruptSignal>,
}

/// Contains all the state that we cannot save in UI directly, because by being
/// passed to polling tasks it would leave the UI in a partially moved state,
/// preventing us from calling methods on it.
struct PollingState {
    watched_command: WatchedCommand,
}

/// Events that are handled in our main UI/IO loop.
pub enum Event {
    CommandOutput(Result<String>),
    KeyPressed(KeyEvent),
    TerminalResized,
    SubcommandCompleted(Result<()>),
    SubcommandForEnvCompleted(Result<EnvVariables>),
}

// TODO: maybe move to operations module
/// Actions that executed subcommands (coming from keybindings) can request.
pub enum RequestedAction {
    /// Continue the execution normally
    Continue,
    /// Reload/rerun the main command, while blocking.
    ReloadWatchedCommand,
    /// Signals that a blocking subcommand has started executing, so we
    /// should block.
    ExecutingBlockingSubcommand,
    /// Signals that a blocking subcommand used to set env variables has
    /// started executing, so we should block.
    ExecutingBlockingSubcommandForEnv,
    /// Exit the application.
    Exit,
}

// TODO: use rust type state pattern
/// Whether or not the app is currently blocking (new events).
/// The app is blocked when blocking commands are executing.
#[derive(Default, Debug)]
enum BlockingState {
    #[default]
    Unblocked,
    BlockedReloadingWatchedCommand,
    BlockedExecutingSubcommand,
    BlockedExecutingSubcommandForEnv,
}

/// Draws the UI. Prevents code duplication, because making this a method would
/// require borrowing self completely, which causes borrow-checker problems.
macro_rules! draw {
    ($self:expr) => {
        $self
            .terminal_manager
            .terminal
            .draw(|frame| $self.state.draw(frame))
    };
}

/// Save all remaining operations, if there are any. Used as macro to prevent
/// borrow-checking problems.
macro_rules! save_remaining_operations {
    ($self:expr, $key:expr, $remaining_index:expr, $operations:expr) => {
        if $remaining_index < $operations.len() {
            $self.remaining_operations = Some(RemainingOperations {
                key: $key,
                remaining_index: $remaining_index,
            });
        }
    };
}

/// Control flow action to be taken after executing operations.
enum ControlFlow {
    Exit,
    Continue,
}

impl UI {
    /// Initiates the user interface.
    pub async fn start(config: Config) -> Result<()> {
        let (ui, polling_state) = UI::new(config).await?;
        ui.run(polling_state).await?;
        Ok(())
    }

    async fn new(config: Config) -> Result<(Self, PollingState)> {
        let terminal_manager = TerminalManager::new()?;

        let env_variables = EnvVariables::generate_initial(config.initial_env_variables).await?;
        let keybindings_str = config.keybindings_parsed.to_string();
        let state = State::new(
            config.header_lines,
            config.fields,
            config.styles,
            keybindings_str,
            env_variables,
        );

        /// The event buffer capacity is restricted to 100 (seems to be a
        /// common default in Tokio) to prevent the message queue from growing
        /// to the point of memory exhaustion.
        const EVENT_BUFFER_CAPACITY: usize = 100;
        let (event_tx, event_rx) = mpsc::channel(EVENT_BUFFER_CAPACITY);

        /// The polling tasks/threads are mostly waiting for signals from the
        /// main event loop, and perform a single action on arrival of a
        /// message. Therefore, the receiving polling tasks should never
        /// receive more than 1 task.
        const POLLING_TASKS_BUFFER_CAPACITY: usize = 1;
        let (reload_tx, reload_rx) = mpsc::channel(POLLING_TASKS_BUFFER_CAPACITY);

        let env_variables = state.get_env();
        let keybindings = Keybindings::from_parsed(config.keybindings_parsed, &env_variables);

        let polling_state = PollingState {
            watched_command: CommandBuilder::new(config.watched_command)
                .blocking()
                .with_output()
                .interruptible(reload_rx)
                .with_env(env_variables.clone()),
        };

        let ui = Self {
            blocking_state: BlockingState::default(),
            terminal_manager,
            state,
            watch_rate: config.watch_rate,
            keybindings: Arc::new(keybindings),
            remaining_operations: None,
            channels: Channels {
                event_tx,
                event_rx,
                reload_tx,
            },
        };

        Ok((ui, polling_state))
    }

    /// Run the main event loop indefinitely until an Exit request is received.
    async fn run(mut self, polling_state: PollingState) -> Result<()> {
        // Launch polling tasks
        tokio::spawn(poll_execute_watched_command(
            polling_state.watched_command,
            self.watch_rate,
            self.channels.event_tx.clone(),
        ));
        tokio::spawn(poll_terminal_events(
            self.keybindings.clone(),
            self.channels.event_tx.clone(),
        ));

        'event_loop: loop {
            draw!(self)?;

            let Some(event) = self.channels.event_rx.recv().await else {
                break 'event_loop;
            };

            // Handle events that are handled the same in every state.
            if let Event::TerminalResized = &event {
                // Reload the UI
                continue 'event_loop;
            }
            // Note: all states also handle Event::CommandOutput very similarly,
            // but taking lines out of event here leaves event in a partially
            // moved state, preventing further usage. Therefore, we tolerate
            // the code duplication below for now.

            match self.blocking_state {
                BlockingState::BlockedReloadingWatchedCommand => {
                    if let Event::CommandOutput(lines) = event {
                        self.state.update_lines(lines?)?;

                        if let ControlFlow::Exit = self.conclude_blocking().await? {
                            break 'event_loop;
                        }
                    }
                }
                BlockingState::BlockedExecutingSubcommand => {
                    match event {
                        Event::CommandOutput(lines) => {
                            // We handle new output lines, but don't exit the
                            // blocking state.
                            self.state.update_lines(lines?)?;
                        }
                        Event::SubcommandCompleted(potential_error) => {
                            potential_error?;

                            if let ControlFlow::Exit = self.conclude_blocking().await? {
                                break 'event_loop;
                            }
                        }
                        _ => {}
                    }
                }
                BlockingState::BlockedExecutingSubcommandForEnv => {
                    match event {
                        Event::CommandOutput(lines) => {
                            // We handle new output lines, but don't exit the
                            // blocking state.
                            self.state.update_lines(lines?)?;
                        }
                        Event::SubcommandForEnvCompleted(new_env_variables) => {
                            self.state.set_env(new_env_variables?).await;

                            if let ControlFlow::Exit = self.conclude_blocking().await? {
                                break 'event_loop;
                            }
                        }
                        _ => {}
                    }
                }
                BlockingState::Unblocked => match event {
                    Event::CommandOutput(lines) => {
                        self.state.update_lines(lines?)?;
                    }
                    Event::KeyPressed(key) => {
                        if let ControlFlow::Exit = self.handle_key_event(key).await? {
                            break 'event_loop;
                        }
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }

    /// Executes the operations associated with a key event, but starting at the
    /// given index in the operations iterator. If we encounter any blocking
    /// operations, we update the remaining operations.
    async fn handle_key_event_given_starting_index(
        &mut self,
        key: KeyEvent,
        starting_index: usize,
    ) -> Result<ControlFlow> {
        if let Some(ops) = self.keybindings.get_operations(&key) {
            for (idx, op) in ops.into_iter().enumerate().skip(starting_index) {
                match op.execute(&mut self.state, &self.channels.event_tx).await? {
                    RequestedAction::Exit => return Ok(ControlFlow::Exit),
                    RequestedAction::ReloadWatchedCommand => {
                        // Send the command execution an interrupt signal
                        // causing the execution to be reloaded.
                        if self.channels.reload_tx.send(InterruptSignal).await.is_err() {
                            return Ok(ControlFlow::Exit);
                        }

                        save_remaining_operations!(self, key, idx + 1, ops);
                        self.blocking_state = BlockingState::BlockedReloadingWatchedCommand;

                        return Ok(ControlFlow::Continue);
                    }
                    RequestedAction::ExecutingBlockingSubcommand => {
                        save_remaining_operations!(self, key, idx + 1, ops);
                        self.blocking_state = BlockingState::BlockedExecutingSubcommand;

                        return Ok(ControlFlow::Continue);
                    }
                    RequestedAction::ExecutingBlockingSubcommandForEnv => {
                        save_remaining_operations!(self, key, idx + 1, ops);
                        self.blocking_state = BlockingState::BlockedExecutingSubcommandForEnv;

                        return Ok(ControlFlow::Continue);
                    }
                    RequestedAction::Continue => {
                        // Redraw the UI between the execution of each
                        // non-blocking operation.
                        draw!(self)?;
                    }
                };
            }

            self.blocking_state = BlockingState::Unblocked;
        }
        Ok(ControlFlow::Continue)
    }

    /// The current blocking state is now over. However, this doesn't guarantee
    /// that we transition to the unblocked state, because we might still have
    /// to execute remaining blocking operations.
    async fn conclude_blocking(&mut self) -> Result<ControlFlow> {
        // Since we are coming from a blocking state, we need to delete all
        // events we received while we were blocking.
        clear_buffer(&mut self.channels.event_rx);

        match self.remaining_operations.take() {
            Some(RemainingOperations {
                key,
                remaining_index,
            }) => {
                // Execute any remaining operations.
                self.handle_key_event_given_starting_index(key, remaining_index)
                    .await
            }
            None => {
                // Given no more remaining operations, we can unblock.
                self.blocking_state = BlockingState::Unblocked;
                Ok(ControlFlow::Continue)
            }
        }
    }

    /// Execute the operations associated with a key event.
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<ControlFlow> {
        self.handle_key_event_given_starting_index(key, 0).await
    }
}

/// The interrupt signal that is sent to the command polling thread when the
/// command execution should be reloaded.
pub struct InterruptSignal;

/// Continuously executes the command in a loop, separated by sleeps of
/// watch_rate duration. Additionally, can be signalled to reload the execution
/// of the command, which simply wakes up this thread sooner.
/// The stdout of successful executions is sent back to the main thread.
async fn poll_execute_watched_command(
    mut watched_command: WatchedCommand,
    watch_rate: Duration,
    event_tx: Sender<Event>,
) {
    loop {
        let start_time = Instant::now();

        let output_lines_result = match watched_command.execute().await {
            Ok(ExecutionResult::Interrupted) => continue,
            Ok(ExecutionResult::Stdout(output_lines)) => Ok(output_lines),
            Err(e) => Err(e),
        };

        if event_tx
            .send(Event::CommandOutput(output_lines_result))
            .await
            .is_err()
        {
            break;
        };

        // If all senders (i.e. the main thread) have been dropped, we abort.
        if watch_rate == Duration::ZERO {
            // Wake up only when notified.
            let WasWoken::ReceivedInterrupt = watched_command.wait_for_interrupt().await else {
                break;
            };
        } else {
            // Wake up at the earliest when notified through recv, or at
            // latest after the watch_rate timeout has passed.
            let timeout = watch_rate.saturating_sub(start_time.elapsed());
            let WasWoken::ReceivedInterrupt = watched_command
                .wait_for_interrupt_within_timeout(timeout)
                .await
            else {
                break;
            };
        }
    }

    log::info!("Shutting down command executor task");
}

/// Continuously listens for terminal-related events, and sends relevant events
/// back to the main thread.
/// For key events, only those that are part of a keybinding are sent.
/// For terminal resizing, we always notify.
async fn poll_terminal_events(keybindings: Arc<Keybindings>, event_tx: Sender<Event>) {
    // TODO: don't listen for events when blocked, isn't displayed anyways
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(CrosstermEvent::Key(key_event))) => {
                if let Ok(key) = key_event.try_into() {
                    if keybindings.get_operations(&key).is_some() {
                        // Ideally, we would send the &Operations directly, instead
                        // of only sending the key event, which the main thread
                        // then as to look-up again in the Keybindings hashmap,
                        // but sending references is impossible/requires a lot of
                        // synchronization overhead.
                        if event_tx.send(Event::KeyPressed(key)).await.is_err() {
                            break;
                        };
                    }
                }
            }
            Some(Ok(CrosstermEvent::Resize(_, _))) => {
                if event_tx.send(Event::TerminalResized).await.is_err() {
                    break;
                };
            }
            _ => {}
        }
    }

    log::info!("Shutting down event listener task");
}

/// Remove all elements from the receiving channel buffer, until is is either
/// empty or was closed by the sender(s).
fn clear_buffer<T>(rx: &mut Receiver<T>) {
    while rx.try_recv().is_ok() {}
}
