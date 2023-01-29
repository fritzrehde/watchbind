mod state;
mod terminal_manager;

pub use state::State;

use crate::command::Command;
use crate::config::Config;
use crate::config::{Key, Keybindings};
use anyhow::Result;
use crossterm::event::{self, Event::Key as CKey};
use std::{
	sync::mpsc::{self, Receiver, Sender, TryRecvError},
	thread,
	time::{Duration, Instant},
};
use terminal_manager::{Terminal, TerminalManager};

pub enum Event {
	KeyPressed(Key),
	CommandOutput(Result<Vec<String>>),
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
	let (wake_tx, wake_rx) = mpsc::sync_channel(1);
	let mut state = State::new(config.styles);

	poll_execute_command(config.watch_rate, config.command, event_tx.clone(), wake_rx);
	poll_key_events(event_tx.clone(), config.keybindings.clone());

	loop {
		terminal.draw(|frame| state.draw(frame))?;

		match event_rx.recv() {
			Ok(Event::CommandOutput(lines)) => state.set_lines(lines?),
			Ok(Event::KeyPressed(key)) => {
				if let Some(ops) = config.keybindings.get_operations(&key) {
					for op in ops.iter() {
						match op.execute(&mut state)? {
							RequestedAction::Exit => return Ok(()),
							RequestedAction::Reload => {
								// TODO: ugly solution
								if let Ok(_) = wake_tx.try_send(()) {
									loop {
										if let Ok(Event::CommandOutput(lines)) = event_rx.recv() {
											state.set_lines(lines?);
											clear_buffer(&mut event_rx);
											break;
										}
									}
								}
							}
							RequestedAction::Unblock => clear_buffer(&mut event_rx),
							RequestedAction::Continue => {}
						};
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
	thread::spawn(move || {
		loop {
			// TODO: write helper function that takes a lambda to measure time difference
			// execute command and time execution
			let start = Instant::now();

			// TODO: kill command process immediately once we receive reload signal
			let lines = command.capture_output();

			let timeout = watch_rate.saturating_sub(start.elapsed());

			if let Ok(_) = reload_rx.try_recv() {
				continue;
			}

			event_tx.send(Event::CommandOutput(lines)).ok();

			// sleep until notified
			if watch_rate == Duration::ZERO {
				reload_rx.recv().ok();
			} else {
				// wake up at latest after watch_rate time
				reload_rx.recv_timeout(timeout).ok();
			}
		}
	});
}

fn poll_key_events(tx: Sender<Event>, keybindings: Keybindings) {
	thread::spawn(move || loop {
		if let CKey(key_event) = event::read().unwrap() {
			let key = key_event.into();
			if keybindings.get_operations(&key).is_some() {
				tx.send(Event::KeyPressed(key)).unwrap();
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
