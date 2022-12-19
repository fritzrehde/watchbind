use crate::config::Config;
use crate::exec::output_lines;
use crate::keybindings::{exec_operation, Key, Operations};
use crate::state::State;
use crate::terminal_manager::{Terminal, TerminalManager};
use anyhow::Result;
use crossterm::event::{self, Event::Key as CKey};
use mpsc::{Receiver, Sender};
use std::{
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

pub enum RequestedAction {
	Continue,
	Reload,
	Block,
	Exit,
}

pub enum Event {
	KeyPressed(Key),
	CommandOutput(Result<Vec<String>>),
	Unblock(Result<()>),
	ExecuteNextCommand,
}

pub fn start(config: Config) -> Result<()> {
	let mut terminal_manager = TerminalManager::new()?;
	let err = run(config, &mut terminal_manager.terminal);
	terminal_manager.restore()?;
	err
}

fn run(config: Config, terminal: &mut Terminal) -> Result<()> {
	// TODO: channels: remove unwraps
	let (event_tx, event_rx) = mpsc::channel();
	let (wake_tx, wake_rx) = mpsc::channel();
	let mut state = State::new(&config.styles);
	let mut operations = Operations::new();
	let mut blocked = false;

	poll_execute_command(
		config.watch_rate.clone(),
		config.command.clone(),
		event_tx.clone(),
		wake_rx,
	);
	poll_key_events(event_tx.clone());

	loop {
		terminal.draw(|frame| state.draw(frame))?;

		match event_rx.recv() {
			Ok(Event::CommandOutput(lines)) => state.set_lines(lines?),
			Ok(Event::KeyPressed(key)) => {
				if !blocked {
					if let Some(new_ops) = config.keybindings.get(&key) {
						operations.add(&new_ops);
					}
					event_tx.send(Event::ExecuteNextCommand).unwrap();
				}
			}
			Ok(Event::Unblock(msg)) => {
				msg?;
				blocked = false;
				event_tx.send(Event::ExecuteNextCommand).unwrap();
			}
			Ok(Event::ExecuteNextCommand) => {
				while !blocked {
					match operations.next() {
						Some(op) => match exec_operation(&op, &mut state, &event_tx)? {
							RequestedAction::Exit => return Ok(()),
							RequestedAction::Reload => wake_tx.send(()).unwrap(),
							RequestedAction::Block => blocked = true,
							RequestedAction::Continue => {}
						},
						None => break,
					}
				}

				// TODO: replace with this once it is stable
				// while !blocked && let Some(op) = operations.pop_front() {
				// 	match exec_operation(&op, &mut state, &event_tx)? {
				// 		RequestedAction::Exit => return Ok(()),
				// 		RequestedAction::Reload => wake_tx.send(()).unwrap(),
				// 		RequestedAction::Block => blocked = true,
				// 		RequestedAction::Continue => {}
				// 	};
				// }
			}
			_ => {}
		};
	}
}

fn poll_execute_command(
	watch_rate: Duration,
	command: String,
	event_tx: Sender<Event>,
	wake_rx: Receiver<()>,
) {
	thread::spawn(move || {
		loop {
			// execute command and time execution
			let before = Instant::now();
			let lines = output_lines(&command);
			let exec_time = Instant::now().duration_since(before);
			let sleep = watch_rate.saturating_sub(exec_time);

			// ignore error that occurs when main thread (and channels) close
			event_tx.send(Event::CommandOutput(lines)).ok();

			// sleep until notified
			if watch_rate == Duration::ZERO {
				wake_rx.recv().ok();
			} else {
				// wake up at latest after watch_rate time
				wake_rx.recv_timeout(sleep).ok();
			}
		}
	});
}

fn poll_key_events(tx: Sender<Event>) {
	thread::spawn(move || loop {
		if let CKey(key) = event::read().unwrap() {
			tx.send(Event::KeyPressed(Key::new(key))).unwrap();
		}
	});
}
