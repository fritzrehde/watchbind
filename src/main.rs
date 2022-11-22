use crate::config::Config;
use crate::events::Events;
use crate::keybindings::Operation;
use crate::style::Styles;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
	io::{self, Error},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use tui::{
	backend::{Backend, CrosstermBackend},
	widgets::{List, ListItem},
	Frame, Terminal,
};

mod config;
mod events;
mod exec;
mod keybindings;
mod style;
mod toml;

fn main() -> Result<(), Error> {
	match config::parse_config() {
		Ok(config) => {
			// setup terminal
			enable_raw_mode()?;
			let mut stdout = io::stdout();
			execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
			let backend = CrosstermBackend::new(stdout);
			let mut terminal = Terminal::new(backend)?;
			terminal.hide_cursor()?;

			// run tui program
			let res = run(&mut terminal, config);

			// restore terminal
			disable_raw_mode()?;
			execute!(
				terminal.backend_mut(),
				LeaveAlternateScreen,
				DisableMouseCapture
			)?;
			terminal.show_cursor()?;

			// print errors to stdout
			match res {
				Err(e) => eprint!("error: {}", e),
				_ => {}
			};
		}
		// print config errors
		Err(e) => eprintln!("error: {}", e),
	};
	Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, config: Config) -> Result<(), io::Error> {
	let mut last_tick = Instant::now();
	let (data_send, data_rcv) = mpsc::channel();
	let (info_send, info_rcv) = mpsc::channel();
	thread::spawn(move || {
		// only execute command once
		if config.watch_rate == Duration::ZERO {
			data_send.send(exec::output_lines(&config.command)).unwrap();
		} else {
			// worker thread that executes command in loop
			loop {
				let before = Instant::now();
				let lines = exec::output_lines(&config.command);
				let exec_time = Instant::now().duration_since(before);
				let sleep = config.watch_rate.saturating_sub(exec_time);
				// ignore error that occurs when main thread (and channels) close
				data_send.send(lines).ok();
				info_rcv.recv_timeout(sleep).ok();
			}
		}
	});
	let mut events = Events::new(data_rcv.recv().unwrap()?);

	// main thread loop
	// TODO: create keyboard input worker thread
	loop {
		match data_rcv.try_recv() {
			Ok(recv) => events.set_items(recv?),
			_ => {}
		};

		terminal.draw(|f| ui(f, &mut events, &config.styles))?;

		let timeout = config
			.tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::ZERO);
		// wait for keyboard input for max time of timeout
		if event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				match keybindings::handle_key(key.code, &config.keybindings, &mut events) {
					Ok(Operation::Exit) => return Ok(()),
					Ok(Operation::Reload) => info_send.send(true).unwrap(),
					Err(e) => return Err(e),
					_ => {}
				};
			}
		}
		if last_tick.elapsed() >= config.tick_rate {
			last_tick = Instant::now();
		}
	}
}

// TODO: simplify
fn ui<B: Backend>(f: &mut Frame<B>, events: &mut Events, styles: &Styles) {
	let items: Vec<ListItem> = events
		.items
		.iter()
		.map(|i| ListItem::new(i.as_ref()))
		.collect();
	// let items = vec![
	// 	ListItem::new("line one"),
	// 	ListItem::new(""),
	// 	ListItem::new("line four"),
	// ];
	let list = List::new(items)
		.style(styles.style)
		.highlight_style(styles.highlight_style);
	f.render_stateful_widget(list, f.size(), &mut events.state);
}
