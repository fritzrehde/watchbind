mod state;
mod terminal_manager;

pub use state::State;

use crate::command::Command;
use crate::config::Config;
use crate::config::{KeyEvent as CKeyEvent, Keybindings};
use anyhow::Result;
use crossterm::event::{self, Event::Key};
use std::{
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use terminal_manager::{Terminal, TerminalManager};

pub enum Event {
    KeyPressed(CKeyEvent),
    CommandOutput(Result<String>),
}

pub enum RequestedAction {
    Continue,
    Reload,
    Unblock,
    Exit,
}

pub fn start(config: Config) -> Result<()> {
    let mut terminal_manager = TerminalManager::new()?;
    let err = run(&mut terminal_manager.terminal, config);
    terminal_manager.restore()?;
    err
}

fn run(terminal: &mut Terminal, config: Config) -> Result<()> {
    // TODO: channels: remove unwraps
    let (event_tx, mut event_rx) = mpsc::channel();
    let (reload_tx, reload_rx) = mpsc::sync_channel(1);
    let mut state = State::new(config.header_lines, config.field_separator, config.styles);

    poll_execute_command(
        config.watch_rate,
        config.command,
        event_tx.clone(),
        reload_rx,
    );
    poll_key_events(event_tx, config.keybindings.clone());

    loop {
        terminal.draw(|frame| state.draw(frame))?;

        // TODO: remove deep match statements
        match event_rx.recv() {
            Ok(Event::CommandOutput(lines)) => state.update_lines(lines?)?,
            Ok(Event::KeyPressed(key)) => {
                if let Some(ops) = config.keybindings.get_operations(&key) {
                    for op in ops {
                        match op.execute(&mut state)? {
                            RequestedAction::Exit => return Ok(()),
                            RequestedAction::Reload => {
                                if reload_tx.try_send(()).is_ok() {
                                    loop {
                                        // TODO: code duplication
                                        if let Ok(Event::CommandOutput(lines)) = event_rx.recv() {
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
}

fn poll_execute_command(
    watch_rate: Duration,
    command: Command,
    event_tx: Sender<Event>,
    reload_rx: Receiver<()>,
) {
    // TODO: don't run command when blocked, isn't displayed anyways
    thread::spawn(move || loop {
        // TODO: write helper function that takes a lambda to measure time difference
        let start = Instant::now();
        let lines = command.capture_output(&reload_rx);
        let timeout = watch_rate.saturating_sub(start.elapsed());
        event_tx.send(Event::CommandOutput(lines)).ok();

        // sleep until notified
        if watch_rate == Duration::ZERO {
            reload_rx.recv().ok();
        } else {
            // wake up at latest after watch_rate time
            reload_rx.recv_timeout(timeout).ok();
        }
    });
}

fn poll_key_events(tx: Sender<Event>, keybindings: Keybindings) {
    thread::spawn(move || loop {
        if let Key(key_event) = event::read().unwrap() {
            if let Ok(key) = key_event.try_into() {
                if keybindings.get_operations(&key).is_some() {
                    tx.send(Event::KeyPressed(key)).unwrap();
                }
            }
        }
    });
}

fn clear_buffer<T>(rx: &mut Receiver<T>) {
    loop {
        if let Err(TryRecvError::Empty) = rx.try_recv() {
            break;
        }
    }
}
