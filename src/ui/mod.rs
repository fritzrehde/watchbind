mod state;
mod terminal_manager;

use crate::command::{AsyncResult, Command};
use crate::config::Config;
use crate::config::{KeyEvent, Keybindings};
use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use terminal_manager::TerminalManager;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub use state::State;

/// All events that are handled in our main UI/IO loop.
pub enum Event {
    CommandOutput(Result<String>),
    KeyPressed(KeyEvent),
    TerminalResized,
}

pub enum RequestedAction {
    /// Continue the execution normally
    Continue,
    /// Reload the command, and block (all other events) while doing so.
    Reload,
    /// Unblock the execution.
    Unblock,
    /// Exit the application.
    Exit,
}

pub struct UI {
    config: Config,
    terminal_manager: TerminalManager,
}

impl UI {
    pub async fn start(config: Config) -> Result<()> {
        UI::new(config)?.run().await?;
        Ok(())
    }

    fn new(config: Config) -> Result<Self> {
        let terminal_manager = TerminalManager::new()?;
        Ok(Self {
            config,
            terminal_manager,
        })
    }

    async fn run(mut self) -> Result<()> {
        let terminal = &mut self.terminal_manager.terminal;

        // TODO: fine tune the buffer size, explain why 100
        let (event_tx, mut event_rx) = mpsc::channel(100);
        // TODO: explain why the buffer is 1 here
        let (reload_tx, reload_rx) = mpsc::channel(1);

        let help_menu_body = self.config.keybindings.to_string();
        let mut state = State::new(
            self.config.header_lines,
            self.config.fields,
            self.config.styles,
            help_menu_body,
        );
        let keybindings = Arc::new(self.config.keybindings);

        tokio::spawn(poll_execute_command(
            self.config.watch_rate,
            self.config.command,
            reload_rx,
            event_tx.clone(),
        ));

        tokio::spawn(poll_terminal_events(event_tx.clone(), keybindings.clone()));

        'main: loop {
            terminal.draw(|frame| state.draw(frame))?;

            // TODO: remove deep match statements
            match event_rx.recv().await {
                Some(Event::CommandOutput(lines)) => {
                    state.update_lines(lines?)?;
                }
                Some(Event::KeyPressed(key)) => {
                    if let Some(ops) = keybindings.get_operations(&key) {
                        for op in ops {
                            match op.execute(&mut state).await? {
                                RequestedAction::Exit => break 'main,
                                RequestedAction::Reload => {
                                    // TODO: refactor
                                    if reload_tx.try_send(()).is_ok() {
                                        loop {
                                            // TODO: code duplication
                                            if let Some(Event::CommandOutput(lines)) =
                                                event_rx.recv().await
                                            {
                                                state.update_lines(lines?)?;
                                                clear_buffer(&mut event_rx);
                                                break;
                                            }
                                        }
                                    }
                                }
                                RequestedAction::Unblock => clear_buffer(&mut event_rx),
                                RequestedAction::Continue => {}
                            };
                            // TODO: code duplication
                            terminal.draw(|frame| state.draw(frame))?;
                        }
                    }
                }
                Some(Event::TerminalResized) => {
                    // Reload the UI
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// Continuously executes the command in a loop, separated by sleeps of
/// watch_rate duration. Additionally, can be signalled to reload the execution
/// of the command, which simply wakes up this thread sooner.
/// The stdout of successful executions is sent back to the main thread.
async fn poll_execute_command(
    watch_rate: Duration,
    command: Command,
    mut reload_rx: Receiver<()>,
    event_tx: Sender<Event>,
) {
    // TODO: don't run command when blocked, isn't displayed anyways
    loop {
        // TODO: write helper function that takes a lambda to measure time difference
        let start = Instant::now();

        let output_lines_result = match command.capture_output(&mut reload_rx).await {
            Ok(AsyncResult::Interrupted) => continue,
            Ok(AsyncResult::Stdout(output_lines)) => Ok(output_lines),
            Err(e) => Err(e),
        };

        let Ok(_) = event_tx
            .send(Event::CommandOutput(output_lines_result))
            .await else {
                break;
            };

        let timeout = watch_rate.saturating_sub(start.elapsed());

        // If all senders (i.e. the main thread) have been dropped, we abort.
        if watch_rate == Duration::ZERO {
            // Wake up only when notified.
            if reload_rx.recv().await.is_none() {
                break;
            }
        } else {
            // Wake up at the earliest when notified through recv, or at
            // latest after the watch_rate timeout has passed.
            if let Ok(None) = tokio::time::timeout(timeout, reload_rx.recv()).await {
                break;
            }
        }
    }

    log::info!("Shutting down command executor task");
}

/// Continuously listens for terminal-related events, and sends relevant events
/// back to the main thread.
/// For key events, only those that are part of a keybinding are sent.
/// For terminal resizing, we always notify.
async fn poll_terminal_events(tx: Sender<Event>, keybindings: Arc<Keybindings>) {
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
                        if tx.send(Event::KeyPressed(key)).await.is_err() {
                            break;
                        };
                    }
                }
            }
            Some(Ok(CrosstermEvent::Resize(_, _))) => {
                if tx.send(Event::TerminalResized).await.is_err() {
                    break;
                };
            }
            _ => {}
        }
    }

    log::info!("Shutting down event listener task");
}

fn clear_buffer<T>(rx: &mut Receiver<T>) {
    loop {
        if let Err(TryRecvError::Empty) = rx.try_recv() {
            break;
        }
    }
}
