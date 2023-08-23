mod state;
mod terminal_manager;

use crate::command::{AsyncResult, Command, ExecutableCommand};
use crate::config::Config;
use crate::config::{KeyEvent, Keybindings};
use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use terminal_manager::TerminalManager;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub use state::State;

pub struct UI {
    terminal_manager: TerminalManager,
    state: State,
    command: Command,
    watch_rate: Duration,
    keybindings: Arc<Keybindings>,
}

/// Events that are handled in our main UI/IO loop.
pub enum Event {
    CommandOutput(Result<String>),
    SubcommandCompleted(Result<()>),
    KeyPressed(KeyEvent),
    TerminalResized,
}

/// Actions that executed subcommands (coming from keybindings) can request.
pub enum RequestedAction {
    /// Continue the execution normally
    Continue,
    /// Reload/rerun the main command, while blocking.
    ReloadCommand,
    /// Execute a blocking subcommand on a worker thread, while blocking.
    ExecuteBlockingSubcommand(ExecutableCommand),
    /// Exit the application.
    Exit,
}

// TODO: use rust type state pattern
/// The app is blocked when blocking commands are executing.
#[derive(Default)]
enum BlockingState {
    #[default]
    Unblocked,
    BlockedReloadingCommand,
    BlockedExecutingSubcommand,
}

impl BlockingState {
    /// When we unblock, we need to delete all events that occurred while
    /// we were blocking.
    fn unblock(&mut self, event_rx: &mut Receiver<Event>) {
        clear_buffer(event_rx);
        *self = BlockingState::Unblocked;
    }
}

/// Draws the UI. Convenient macro to prevent having to require borrowing
/// self completely, which causes borrow-checker problems.
macro_rules! draw {
    ($self:expr) => {
        $self
            .terminal_manager
            .terminal
            .draw(|frame| $self.state.draw(frame))
    };
}

impl UI {
    /// Initiates the user interface.
    pub async fn start(config: Config) -> Result<()> {
        UI::new(config)?.run().await?;
        Ok(())
    }

    fn new(config: Config) -> Result<Self> {
        let terminal_manager = TerminalManager::new()?;

        let help_menu_body = config.keybindings.to_string();
        let state = State::new(
            config.header_lines,
            config.fields,
            config.styles,
            help_menu_body,
        );

        Ok(Self {
            terminal_manager,
            state,
            command: config.command,
            watch_rate: config.watch_rate,
            keybindings: Arc::new(config.keybindings),
        })
    }

    async fn run(mut self) -> Result<()> {
        // TODO: fine tune the buffer size, explain why 100
        let (event_tx, mut event_rx) = mpsc::channel(100);
        // TODO: explain why the buffer is 1 here
        let (reload_tx, reload_rx) = mpsc::channel(1);
        let (subcommand_tx, subcommand_rx) = mpsc::channel(1);

        let mut blocking_state = BlockingState::default();

        // Launch polling tasks
        tokio::spawn(poll_execute_command(
            self.watch_rate,
            self.command,
            reload_rx,
            event_tx.clone(),
        ));
        tokio::spawn(poll_execute_subcommands(subcommand_rx, event_tx.clone()));
        tokio::spawn(poll_terminal_events(
            self.keybindings.clone(),
            event_tx.clone(),
        ));

        'event_loop: loop {
            draw!(self)?;

            match blocking_state {
                BlockingState::BlockedReloadingCommand => match event_rx.recv().await {
                    Some(Event::CommandOutput(lines)) => {
                        self.state.update_lines(lines?)?;
                        blocking_state.unblock(&mut event_rx);
                    }
                    Some(Event::TerminalResized) => {} // Reload the UI,
                    _ => {}
                },
                BlockingState::BlockedExecutingSubcommand => match event_rx.recv().await {
                    // When we are blocked waiting for the subcommand to finish
                    // executing, we still process the stdout of the main
                    // command, but crucially don't exit our current state.
                    Some(Event::CommandOutput(lines)) => {
                        self.state.update_lines(lines?)?;
                    }
                    Some(Event::SubcommandCompleted(result)) => {
                        result?;
                        blocking_state.unblock(&mut event_rx);
                    }
                    Some(Event::TerminalResized) => {} // Reload the UI,
                    _ => {}
                },
                BlockingState::Unblocked => {
                    match event_rx.recv().await {
                        Some(Event::CommandOutput(lines)) => {
                            self.state.update_lines(lines?)?;
                        }
                        Some(Event::KeyPressed(key)) => {
                            if let Some(ops) = self.keybindings.get_operations(&key) {
                                // TODO: idea: pop from the front, so blocking commands can take the rest to execute after finishing blocking
                                for op in ops {
                                    log::debug!("Executing op: {}", op);

                                    match op.execute(&mut self.state).await? {
                                        RequestedAction::Exit => break 'event_loop,
                                        RequestedAction::ReloadCommand => {
                                            // Send the command execution an interrupt
                                            // signal causing the execution to be
                                            // reloaded.
                                            if reload_tx.send(InterruptSignal).await.is_err() {
                                                break 'event_loop;
                                            }
                                            blocking_state = BlockingState::BlockedReloadingCommand;
                                            // TODO: by leaving this for loop, we are ignoring/forgetting to execute the remaining operations => save remaining operations for execution after we finished blocking
                                            continue 'event_loop;
                                        }
                                        RequestedAction::ExecuteBlockingSubcommand(subcommand) => {
                                            if subcommand_tx.send(subcommand).await.is_err() {
                                                break 'event_loop;
                                            }
                                            blocking_state =
                                                BlockingState::BlockedExecutingSubcommand;
                                            // TODO: by leaving this for loop, we are ignoring/forgetting to execute the remaining operations => save remaining operations for execution after we finished blocking
                                            continue 'event_loop;
                                        }
                                        RequestedAction::Continue => {}
                                    };

                                    // Redraw the UI after each operation's execution.
                                    draw!(self)?;
                                }
                            }
                        }
                        Some(Event::TerminalResized) => {} // Reload the UI,
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

/// The interrupt signal that is sent to the command polling thread when the
/// command execution should be reloaded.
pub struct InterruptSignal;

/// Continuously executes the command in a loop, separated by sleeps of
/// watch_rate duration. Additionally, can be signalled to reload the execution
/// of the command, which simply wakes up this thread sooner.
/// The stdout of successful executions is sent back to the main thread.
async fn poll_execute_command(
    watch_rate: Duration,
    command: Command,
    mut reload_rx: Receiver<InterruptSignal>,
    event_tx: Sender<Event>,
) {
    // TODO: don't run command when blocked, isn't displayed anyways
    loop {
        let start_time = Instant::now();

        let output_lines_result = match command.capture_output(&mut reload_rx).await {
            Ok(AsyncResult::Interrupted) => continue,
            Ok(AsyncResult::Stdout(output_lines)) => Ok(output_lines),
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
            if reload_rx.recv().await.is_none() {
                break;
            }
        } else {
            // Wake up at the earliest when notified through recv, or at
            // latest after the watch_rate timeout has passed.
            let timeout = watch_rate.saturating_sub(start_time.elapsed());
            if let Ok(None) = tokio::time::timeout(timeout, reload_rx.recv()).await {
                break;
            }
        }
    }

    log::info!("Shutting down command executor task");
}

/// Continuously waits for commands and executes them on arrival.
/// Sends potential errors during execution back to the main thread.
async fn poll_execute_subcommands(
    mut new_command_rx: Receiver<ExecutableCommand>,
    event_tx: Sender<Event>,
) {
    // Wait for a new task.
    while let Some(mut command) = new_command_rx.recv().await {
        // Execute command
        let result = command.execute().await;

        if event_tx
            .send(Event::SubcommandCompleted(result))
            .await
            .is_err()
        {
            break;
        };
    }

    log::info!("Shutting down subcommand executor task");
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
