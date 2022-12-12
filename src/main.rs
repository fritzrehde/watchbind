mod config;
mod stateful_list;
mod exec;
mod keybindings;
mod style;
mod terminal_manager;

use crate::config::Config;
use crate::stateful_list::StatefulList;
use crossterm::event::{self, Event};
use std::{
	io::{self, Error},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use terminal_manager::{TerminalManager, Terminal};

fn main() -> Result<(), Error> {
	match config::parse_config() {
		// print config errors
		Err(e) => eprintln!("error: {}", e),
		Ok(config) => {
			let mut terminal_manager = TerminalManager::new()?;
			let exit = run(config, &mut terminal_manager.terminal);
			terminal_manager.restore()?;
			// print errors to stdout
			if let Err(e) = exit {
				eprint!("error: {}", e);
			}
		}
	};
	Ok(())
}

fn run(config: Config, terminal: &mut Terminal) -> Result<(), io::Error> {
	let mut last_tick = Instant::now();
	let (data_send_channel, data_rcv_channel) = mpsc::channel();
	let (info_send_channel, info_rcv_channel) = mpsc::channel();

	thread::spawn(move || {
		// worker thread that executes command in loop
		loop {
			// execute command and time execution
			let before = Instant::now();
			let lines = exec::output_lines(&config.command);
			let exec_time = Instant::now().duration_since(before);
			let sleep = config.watch_rate.saturating_sub(exec_time);

			// ignore error that occurs when main thread (and channels) close
			data_send_channel.send(lines).ok();

			// sleep until notified
			if config.watch_rate == Duration::ZERO {
				info_rcv_channel.recv().ok();
			} else {
				// wake up at latest after watch_rate time
				info_rcv_channel.recv_timeout(sleep).ok();
			}
		}
	});
	let lines = data_rcv_channel.recv().unwrap()?;
	let mut state = StatefulList::new(lines, &config.styles);

	// main thread loop
	// TODO: create keyboard input worker thread
	loop {
		match data_rcv_channel.try_recv() {
			Ok(lines) => state.set_lines(lines?),
			_ => {}
		};

		// TODO: state shouldn't draw itself, others should "draw state to frame"
		terminal.draw(|frame| state.draw(frame))?;

		let timeout = config
			.tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::ZERO);

		// wait for keyboard input for max time of timeout
		if event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				if let false = keybindings::handle_key(
					key.code,
					&config.keybindings,
					&mut state,
					&info_send_channel,
				)? {
					// exit program
					return Ok(());
				}
			}
		}
		if last_tick.elapsed() >= config.tick_rate {
			last_tick = Instant::now();
		}
	}
}
