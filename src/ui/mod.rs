mod state;
mod terminal_manager;

use crate::command::{AsyncResult, Command};
use crate::config::Config;
use crate::config::{KeyEvent, Keybindings};
use anyhow::Result;
use crossterm::event::Event::Key;
use crossterm::event::EventStream;
use futures::{future::FutureExt, StreamExt};
use std::time::{Duration, Instant};
use terminal_manager::TerminalManager;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub use state::State;

pub enum Event {
    KeyPressed(KeyEvent),
    CommandOutput(Result<String>),
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

        tokio::task::spawn(poll_execute_command(
            self.config.watch_rate,
            self.config.command,
            reload_rx,
            event_tx.clone(),
        ));

        tokio::task::spawn(poll_terminal_events(
            event_tx.clone(),
            self.config.keybindings.clone(),
        ));

        'main: loop {
            terminal.draw(|frame| state.draw(frame))?;

            // TODO: remove deep match statements
            match event_rx.recv().await {
                Some(Event::CommandOutput(lines)) => {
                    state.update_lines(lines?)?;
                }
                Some(Event::KeyPressed(key)) => {
                    if let Some(ops) = self.config.keybindings.get_operations(&key) {
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
                _ => {}
            }
        }

        Ok(())
    }
}

/// Continiously executes the command in a loop, separated by sleeps of
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

/// Continiously listens for terminal-related events (key presses and resizings).
/// Sends relevant events back to the main thread.
async fn poll_terminal_events(tx: Sender<Event>, keybindings: Keybindings) {
    // TODO: don't listen for events when blocked, isn't displayed anyways
    let mut reader = EventStream::new();

    loop {
        let event = reader.next().fuse();

        // TODO: optimize, only send the keys we have mapped in the keybindings. might as well send the operation to be performed directly

        if let Some(Ok(Key(key_event))) = event.await {
            log::debug!("Key pressed: {:?}", key_event);

            if let Ok(key) = key_event.try_into() {
                if keybindings.get_operations(&key).is_some() {
                    let Ok(_) = tx.send(Event::KeyPressed(key)).await else {
                        break;
                    };
                }
            }
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
