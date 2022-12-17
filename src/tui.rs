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
	let mut state = State::new(Vec::new(), &config.styles);

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
				if !handle_key(key, &config.keybindings, &mut state, &wake_tx)? {
					// exit program
					return Ok(());
				}
			}
			Ok(Event::CommandOutput(lines)) => state.set_lines(lines?),
			_ => {},
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
