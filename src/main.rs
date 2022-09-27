use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
	io,
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};
use tui::{
	backend::{Backend, CrosstermBackend},
	widgets::{List, ListItem},
	Frame, Terminal,
};
use crate::config::Config;
use crate::style::Styles;
use crate::events::Events;

mod config;
mod toml;
mod style;
mod keys;
mod events;
mod exec;

fn main() -> Result<(), io::Error> {
	let config = config::parse_config()?;
	// println!("{:?}", config);

	// TODO: possibly remove for speed reasons
	// test command once and exit on failure
	// match exec::output_lines(&config.command) {
	// 	Err(e) => {
	// 		print!("{}", e);
	// 		return Ok(());
	// 	}
	// 	_ => {}
	// };

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

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
		Err(e) => print!("{}", e),
		_ => {}
	};

	Ok(())
}



fn run<B: Backend>(terminal: &mut Terminal<B>, config: Config) -> Result<(), io::Error> {
	let mut last_tick = Instant::now();
	let (tx, rx) = mpsc::channel();
	thread::spawn(move || {
		// worker thread that executes command in loop
		loop {
			tx.send(exec::output_lines(&config.command)).unwrap();
			if config.watch_rate == Duration::ZERO {
				// only execute command once
				break;
			}
			thread::sleep(config.watch_rate);
		}
	});
	let mut events = Events::new(rx.recv().unwrap()?);

	// main thread loop
	// TODO: create keyboard input worker thread
	loop {
		match rx.try_recv() {
			Ok(recv) => events.set_items(recv?),
			_ => {}
		};

		terminal.draw(|f| ui(f, &mut events, &config.styles))?;

		let timeout = config.tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::ZERO);
		// wait for keyboard input for max time of timeout
		if event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				match keys::handle_key(key.code, &config.keybindings, &mut events) {
					// TODO: use sth more elegant than bool return type
					Ok(false) => return Ok(()),
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
