use crate::config::Config;
use crate::exec::output_lines;
use crate::keybindings::{get_key_operations, exec_operation, Operation};
use crate::state::State;
use crate::terminal_manager::{Terminal, TerminalManager};
use anyhow::Result;
use crossterm::event::{self, Event::Key, KeyCode};
use mpsc::{Receiver, Sender};
use std::{
	sync::mpsc,
	thread,
	time::{Duration, Instant},
	collections::VecDeque,
};

pub enum RequestedAction {
	Continue,
	Reload,
	Block,
	Exit,
}

pub enum Event {
	KeyPressed(KeyCode),
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
	let (event_tx, event_rx) = mpsc::channel();
	let (wake_tx, wake_rx) = mpsc::channel();
	let mut state = State::new(&config.styles);
	let mut blocked = false;
	let mut operations = VecDeque::<Operation>::new();

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
					operations.append(&mut VecDeque::from(get_key_operations(key, &config.keybindings)));
					event_tx.send(Event::ExecuteNextCommand).unwrap();
				}
			}
			Ok(Event::Unblock(msg)) => {
				msg?;
				blocked = false;
				event_tx.send(Event::ExecuteNextCommand).unwrap();
			},
			Ok(Event::ExecuteNextCommand) => {
				while !blocked {
					match operations.pop_front() {
						Some(op) => match exec_operation(&op, &mut state, &event_tx)? {
							RequestedAction::Exit => return Ok(()),
							// reload input by waking up thread
							RequestedAction::Reload => wake_tx.send(()).unwrap(),
							RequestedAction::Block => blocked = true,
							RequestedAction::Continue => {},
						},
						None => break,
					}
				}
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
	thread::spawn(move || {
		loop {
			// TODO: remove unwraps
			if let Key(key) = event::read().unwrap() {
				tx.send(Event::KeyPressed(key.code)).unwrap();
			}
		}
	});
}
