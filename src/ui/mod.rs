mod state;
mod terminal_manager;

use anyhow::Result;
use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyEvent as CrosstermKeyEvent, KeyEventKind,
};
use futures::{future::FutureExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use terminal_manager::Tui;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::config::{Config, KeyEvent, Keybindings};
use crate::utils::command::{
    Blocking, CommandBuilder, ExecutionResult, Interruptible, WasWoken, WithEnv, WithOutput,
};

pub use self::state::State;
pub use self::state::{EnvVariable, EnvVariables};

pub type WatchedCommand = CommandBuilder<Blocking, WithEnv, WithOutput, Interruptible>;

pub struct UI {
    blocking_state: BlockingState,
    tui: Tui,
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

    // We don't store the receivers for these channels,
    // because their ownership is passed to the polling tasks.
    reload_tx: Sender<InterruptSignal>,
    polling_tx: Sender<PollingCommand>,
}

/// Contains all the state that we cannot save in UI directly, because by being
/// passed to polling tasks it would leave the UI in a partially moved state,
/// preventing us from calling methods on it.
struct PollingState {
    /// The command of which the output is 'watched'.
    watched_command: WatchedCommand,
    polling_rx: Receiver<PollingCommand>,
}

/// Events that are handled in our main UI/IO loop.
pub enum Event {
    /// The output of a completed command.
    CommandOutput(Result<String>),
    /// A key has been pressed.
    KeyPressed(KeyEvent),
    /// The terminal has been resized.
    TerminalResized,
    /// A subcommand has finished executing.
    SubcommandCompleted(Result<()>),
    /// The output of a completed subcommand, that should now be set to an
    /// env variable.
    SubcommandForEnvCompleted(Result<EnvVariables>),
    /// A TUI subcommand has finished executing.
    TUISubcommandCompleted(Result<()>),
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
    /// Signals that watchbind's TUI needs to be hidden so the TUI subcommand
    /// can be displayed. Notifies event's sender once TUI is finally hidden.
    ExecutingTUISubcommand(Sender<()>),
    /// Exit the application.
    Exit,
}

// TODO: use rust type state pattern
// TODO: split into Unblocked|Blocked and then reason why blocked

/// Whether or not the app is currently blocking (new events).
/// The app is blocked when blocking commands are executing.
#[derive(Default, Debug)]
enum BlockingState {
    #[default]
    Unblocked,
    BlockedReloadingWatchedCommand,
    BlockedExecutingSubcommand,
    BlockedExecutingSubcommandForEnv,
    BlockedExecutingTUISubcommand,
}

/// Clean wrapper around draw() which prevents borrow-checking problems caused
/// by mutably borrowing self.
macro_rules! draw {
    ($self:expr) => {
        draw(&mut $self.tui, &mut $self.state)
    };
}

/// Draw state to a TUI.
fn draw(tui: &mut Tui, state: &mut State) -> Result<()> {
    tui.draw(|frame| state.draw(frame))?;
    Ok(())
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
        let terminal_manager = Tui::new()?;

        // Create `State`.
        let keybindings_str = config.keybindings_parsed.to_string();
        let mut state = State::new(
            config.header_lines,
            config.fields,
            config.styles,
            keybindings_str,
            EnvVariables::new(),
        );
        state
            .generate_initial_env_vars(config.initial_env_ops)
            .await?;

        // TODO: room for optimization: we can probably get away with much smaller buffer sizes for some of our channels

        /// The channel buffer capacity is restricted to 100 (seems to be a
        /// common default in Tokio) to prevent the message queue from growing
        /// to the point of memory exhaustion.
        const TOKIO_DEFAULT_CHANNEL_BUFFER_CAPACITY: usize = 100;

        let (event_tx, event_rx) = mpsc::channel(TOKIO_DEFAULT_CHANNEL_BUFFER_CAPACITY);
        let (reload_tx, reload_rx) = mpsc::channel(TOKIO_DEFAULT_CHANNEL_BUFFER_CAPACITY);
        let (polling_tx, polling_rx) = mpsc::channel(TOKIO_DEFAULT_CHANNEL_BUFFER_CAPACITY);

        let env_variables = state.get_env();
        let keybindings = Keybindings::from_parsed(config.keybindings_parsed, &env_variables);

        let polling_state = PollingState {
            watched_command: CommandBuilder::new(config.watched_command)
                .blocking()
                .with_output()
                .interruptible(reload_rx)
                .with_env(env_variables.clone()),
            polling_rx,
        };

        let ui = Self {
            blocking_state: BlockingState::default(),
            tui: terminal_manager,
            state,
            watch_rate: config.watch_rate,
            keybindings: Arc::new(keybindings),
            remaining_operations: None,
            channels: Channels {
                event_tx,
                event_rx,
                reload_tx,
                polling_tx,
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
            polling_state.polling_rx,
        ));

        'event_loop: loop {
            // Don't draw our own TUI when it is hidden while executing another TUI.
            match self.blocking_state {
                BlockingState::BlockedExecutingTUISubcommand => {}
                _ => {
                    draw!(self)?;
                }
            };

            let Some(event) = self.channels.event_rx.recv().await else {
                // Event channel has been closed.
                break 'event_loop;
            };

            // Handle events that are handled the same in every state.
            if let Event::TerminalResized = &event {
                // Reload the UI.
                continue 'event_loop;
            }

            // Note: all states also handle Event::CommandOutput very similarly,
            // but taking lines out of event here leaves event in a partially
            // moved state, preventing further usage. Therefore, we tolerate
            // the code duplication below for now.

            match self.blocking_state {
                BlockingState::Unblocked => match event {
                    Event::CommandOutput(lines) => {
                        self.state.update_lines(lines?)?;
                    }
                    Event::KeyPressed(key) => {
                        if let ControlFlow::Exit = self.handle_key_event(key).await? {
                            break 'event_loop;
                        }
                    }
                    // Already handled before.
                    Event::TerminalResized => {}
                    // Currently not blocking, so should never receive completed subcommand events.
                    Event::SubcommandCompleted(_)
                    | Event::SubcommandForEnvCompleted(_)
                    | Event::TUISubcommandCompleted(_) => {}
                },
                BlockingState::BlockedExecutingTUISubcommand => match event {
                    Event::TUISubcommandCompleted(potential_error) => {
                        potential_error?;

                        // Remove temporary env vars that were added just for execution.
                        self.state.remove_cursor_and_selected_lines_from_env().await;

                        self.tui.restore()?;
                        log::info!("Watchbind's TUI is shown.");

                        // Resume listening to terminal events in our TUI.
                        self.channels
                            .polling_tx
                            .send(PollingCommand::Listen)
                            .await?;

                        if let ControlFlow::Exit = self.conclude_blocking().await? {
                            break 'event_loop;
                        }
                    }
                    // Our TUI is disabled, so we can't display new output anyways.
                    Event::CommandOutput(_) => {}
                    // Already handled before.
                    Event::TerminalResized => {}
                    // TUI should not be interactive while blocking.
                    Event::KeyPressed(_) => {}
                    // Currently not blocking, so should never receive completed subcommand events.
                    Event::SubcommandCompleted(_) | Event::SubcommandForEnvCompleted(_) => {}
                },
                BlockingState::BlockedReloadingWatchedCommand => match event {
                    Event::CommandOutput(lines) => {
                        // TODO: is called from async context, should be put in spawn_blocking
                        self.state.update_lines(lines?)?;

                        if let ControlFlow::Exit = self.conclude_blocking().await? {
                            break 'event_loop;
                        }
                    }
                    // Already handled before.
                    Event::TerminalResized => {}
                    // TUI should not be interactive while blocking.
                    Event::KeyPressed(_) => {}
                    // Currently not waiting for any blocking subcommand to complete.
                    Event::SubcommandCompleted(_)
                    | Event::SubcommandForEnvCompleted(_)
                    | Event::TUISubcommandCompleted(_) => {}
                },
                BlockingState::BlockedExecutingSubcommand => match event {
                    Event::CommandOutput(lines) => {
                        // TODO: it's up for discussion if we really want this behaviour, need to find use-cases against first

                        // We handle new output lines, but don't exit the
                        // blocking state.
                        self.state.update_lines(lines?)?;
                    }
                    Event::SubcommandCompleted(potential_error) => {
                        potential_error?;

                        // Remove temporary env vars that were added just for execution.
                        self.state.remove_cursor_and_selected_lines_from_env().await;

                        if let ControlFlow::Exit = self.conclude_blocking().await? {
                            break 'event_loop;
                        }
                    }
                    // Already handled before.
                    Event::TerminalResized => {}
                    // TUI should not be interactive while blocking.
                    Event::KeyPressed(_) => {}
                    // Currently not waiting for any blocking subcommand to complete.
                    Event::SubcommandForEnvCompleted(_) | Event::TUISubcommandCompleted(_) => {}
                },
                BlockingState::BlockedExecutingSubcommandForEnv => match event {
                    Event::CommandOutput(lines) => {
                        // We handle new output lines, but don't exit the
                        // blocking state.
                        self.state.update_lines(lines?)?;
                    }
                    Event::SubcommandForEnvCompleted(new_env_variables) => {
                        // Remove temporary env vars that were added just for execution.
                        self.state.remove_cursor_and_selected_lines_from_env().await;

                        self.state.set_envs(new_env_variables?).await;

                        if let ControlFlow::Exit = self.conclude_blocking().await? {
                            break 'event_loop;
                        }
                    }
                    // Already handled before.
                    Event::TerminalResized => {}
                    // TUI should not be interactive while blocking.
                    Event::KeyPressed(_) => {}
                    // Currently not waiting for any blocking subcommand to complete.
                    Event::SubcommandCompleted(_) | Event::TUISubcommandCompleted(_) => {}
                },
            };
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
                match op
                    .execute(&mut self.state, &self.channels.event_tx, &key)
                    .await?
                {
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
                    RequestedAction::ExecutingTUISubcommand(tui_hidden_tx) => {
                        self.pause_terminal_events_polling().await?;

                        self.tui.hide()?;
                        tui_hidden_tx.send(()).await?;
                        log::info!("Watchbind's TUI has been hidden.");

                        save_remaining_operations!(self, key, idx + 1, ops);
                        self.blocking_state = BlockingState::BlockedExecutingTUISubcommand;

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

    /// Tells the terminal event listener thread to stop polling.
    async fn pause_terminal_events_polling(&self) -> Result<()> {
        // Create channels for waiting until polling has actually been paused.
        let (polling_paused_tx, mut polling_paused_rx) = mpsc::channel(1);

        self.channels
            .polling_tx
            .send(PollingCommand::Pause(polling_paused_tx))
            .await?;

        // Wait until polling has actually been paused.
        let _ = polling_paused_rx.recv().await;

        Ok(())
    }

    /// Remove all elements from the events channel.
    fn clear_events_channel(&mut self) {
        clear_buffer(&mut self.channels.event_rx);
    }

    /// The current blocking state is now over. However, this doesn't guarantee
    /// that we transition to the unblocked state, because we might still have
    /// to execute remaining blocking operations.
    async fn conclude_blocking(&mut self) -> Result<ControlFlow> {
        // Since we are coming from a blocking state, we need to delete all
        // events we received while we were blocking.
        self.clear_events_channel();

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

/// A command sent to a polling thread.
enum PollingCommand {
    /// Continue listening/polling for terminal events.
    Listen,
    /// Pause listening/polling for terminal events. Notifies event's sender
    /// once polling has actually been paused.
    Pause(Sender<PollingPaused>),
}

/// A message, sent via a channel, that the polling has been paused.
struct PollingPaused;

/// Continuously listens for terminal-related events, and sends relevant events
/// back to the main thread.
/// For key events, only those that are part of a keybinding are sent.
/// For terminal resizing, we always notify.
async fn poll_terminal_events(
    keybindings: Arc<Keybindings>,
    event_tx: Sender<Event>,
    mut polling_rx: Receiver<PollingCommand>,
) {
    'main_loop: loop {
        // Poll terminal events until instructed to pause.
        let polling_paused_tx = {
            // Recreate the EventStream everytime we start polling terminal
            // events (again).
            let mut terminal_event_reader = EventStream::new();

            'polling_loop: loop {
                tokio::select! {
                    // Wait for receival of a polling command from main event loop thread.
                    polling = polling_rx.recv() => match polling {
                        Some(PollingCommand::Pause(polling_paused_tx)) => break 'polling_loop polling_paused_tx,
                        // Currently already listening for terminal events.
                        Some(PollingCommand::Listen) => continue 'polling_loop,
                        // Channel has been closed.
                        None => break 'main_loop,
                    },
                    // Wait for a terminal event.
                    Some(Ok(event)) = terminal_event_reader.next().fuse() => match event {
                        // Only react to key press, otherwise we might react
                        // to both key press and key release.
                        CrosstermEvent::Key(key_event @ CrosstermKeyEvent { kind: KeyEventKind::Press, .. }) => {
                            if let Ok(key) = key_event.try_into() {
                                log::info!("Key pressed: {}", key);

                                if keybindings.get_operations(&key).is_some() {
                                    // Ideally, we would send the &Operations directly, instead
                                    // of only sending the key event, which the main thread
                                    // then has to look-up again in the Keybindings hashmap,
                                    // but sending references is infeasible (a lot of
                                    // synchronization overhead).
                                    if event_tx.send(Event::KeyPressed(key)).await.is_err() {
                                        break 'main_loop;
                                    };
                                }
                            }
                        }
                        CrosstermEvent::Resize(_, _) => {
                            if event_tx.send(Event::TerminalResized).await.is_err() {
                                break 'main_loop;
                            };
                        }
                        _ => continue 'polling_loop,
                    }
                }
            }
        };

        log::info!("Terminal event listener has been paused.");

        // Notify sender thread that polling has been paused.
        let _ = polling_paused_tx.send(PollingPaused).await;

        // Wait until another Listen command is received.
        'wait_for_listen: while let Some(polling) = polling_rx.recv().await {
            if let PollingCommand::Listen = polling {
                break 'wait_for_listen;
            }
        }

        log::info!("Terminal event listener is listening again.");
    }

    log::info!("Shutting down terminal event listener task");
}

// TODO: implement a trait on rx so we can call this directly on rx

/// Remove all elements from a receiving channel buffer, until it is either
/// empty or was closed by the sender(s).
fn clear_buffer<T>(rx: &mut Receiver<T>) {
    while rx.try_recv().is_ok() {}
}
