use crate::config::Config;
use crate::exec::output_lines;
use crate::keybindings::handle_key;
use crate::state::State;
use crate::terminal_manager::{Terminal, TerminalManager};
use anyhow::Result;
use crossterm::event::{self, Event::Key, KeyCode};
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

enum Event {
	KeyPressed(KeyCode),
	CommandOutput(Result<Vec<String>>),
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

	poll_execute_command(
		config.watch_rate.clone(),
		config.command.clone(),
		&event_tx,
		wake_rx,
	);
	poll_key_events(&event_tx);

	loop {
		terminal.draw(|frame| state.draw(frame))?;

		match event_rx.try_recv() {
			Ok(Event::KeyPressed(key)) => {
				for requested_state in handle_key(key, &config.keybindings, &mut state)?.iter() {
					match requested_state {
						RequestedAction::Exit => return Ok(()),
						// reload input by waking up thread
						RequestedAction::Reload => wake_tx.send(()).unwrap(),
						RequestedAction::Block => {}
						RequestedAction::Continue => {}
					}
				}
			}
			// TODO: possible inefficiency: blocks (due to blocking subshell command), but continues executing and sending CommandOutputs => old set_lines will be called even though new ones are available
			// TODO: solution: enter blocking state where all key events are ignored but command output is still handled (implement with another channel?)
			Ok(Event::CommandOutput(lines)) => state.set_lines(lines?),
			_ => {}
		};
	}
}

fn poll_execute_command(
	watch_rate: Duration,
	command: String,
	event_tx: &Sender<Event>,
	wake_rx: Receiver<()>,
) {
	let event_tx = event_tx.clone();

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

fn poll_key_events(tx: &Sender<Event>) {
	let tx = tx.clone();

	thread::spawn(move || {
		loop {
			// TODO: remove unwraps
			if let Key(key) = event::read().unwrap() {
				tx.send(Event::KeyPressed(key.code)).unwrap();
			}
		}
	});
}
