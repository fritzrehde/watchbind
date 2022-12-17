use crate::config::Config;
use crate::state::State;
use crate::terminal_manager::{Terminal, TerminalManager};
use crate::exec::output_lines;
use crate::keybindings;
use crossterm::event::{self, Event};
use std::{
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use anyhow::Result;

pub fn start(config: Config) -> Result<()> {
	let mut terminal_manager = TerminalManager::new()?;
	let tmp_err = run(config, &mut terminal_manager.terminal);
	terminal_manager.restore()?;
	tmp_err
}

fn run(config: Config, terminal: &mut Terminal) -> Result<()> {
	let mut last_tick = Instant::now();
	let (data_send_channel, data_rcv_channel) = mpsc::channel();
	let (info_send_channel, info_rcv_channel) = mpsc::channel();

	// TODO move to own function
	thread::spawn(move || {
		// worker thread that executes command in loop
		loop {
			// execute command and time execution
			let before = Instant::now();
			let lines = output_lines(&config.command);
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
	let mut state = State::new(lines, &config.styles);

	// TODO: move to own function
	// main thread loop
	// TODO: create keyboard input worker thread
	loop {
		match data_rcv_channel.try_recv() {
			Ok(lines) => state.set_lines(lines?),
			_ => {}
		};

		// TODO: state shouldn't draw itself, others should "draw state on/and frame"
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
