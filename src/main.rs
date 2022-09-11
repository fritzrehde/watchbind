use crate::color::Styles;
use crate::events::Events;
use crate::keys::Command;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
	collections::HashMap,
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

mod cli;
mod color;
mod events;
mod exec;
mod keys;

const TICK_RATE: u64 = 250; // tui repaint interval in ms

fn main() -> Result<(), io::Error> {
	// parse args and options
	let args = cli::parse_args();
	// let interval: f64 = *args.get_one("interval").unwrap_or(&DEFAULT_INTERVAL); // TODO: use default duration directly
	let interval: f64 = *args.get_one("interval").unwrap();
	let tick_rate = Duration::from_millis(TICK_RATE);
	let watch_rate = Duration::from_secs_f64(interval);
	let keybindings = keys::parse_bindings(args.value_of("keybindings").unwrap_or(""))?; // TODO: replace with get_many
	let command: String = args
		.values_of("command")
		.unwrap()
		.collect::<Vec<&str>>()
		.join(" "); // TODO: deprecated, replace with get_many()
	let styles: Styles = color::parse_colors(
		args.value_of("fg"),
		args.value_of("bg"),
		args.value_of("fg+"),
		args.value_of("bg+"),
	);

	// test command once and exit on failure
	match exec::output_lines(&command) {
		Err(e) => {
			print!("{}", e);
			return Ok(());
		}
		_ => {}
	};

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// run tui program
	let res = run(
		&mut terminal,
		&keybindings,
		command,
		styles,
		tick_rate,
		watch_rate,
	);

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

fn run<B: Backend>(
	terminal: &mut Terminal<B>,
	keybindings: &HashMap<KeyCode, Command>,
	command: String,
	styles: Styles,
	tick_rate: Duration,
	watch_rate: Duration,
) -> Result<(), io::Error> {
	let mut last_tick = Instant::now();
	let (tx, rx) = mpsc::channel();
	thread::spawn(move || {
		// worker thread that executes command in loop
		loop {
			tx.send(exec::output_lines(&command)).unwrap();
			if watch_rate == Duration::ZERO {
				// only execute command once
				break;
			}
			thread::sleep(watch_rate);
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

		terminal.draw(|f| ui(f, &mut events, &styles))?;

		let timeout = tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::ZERO);
		// wait for keyboard input for max time of timeout
		if event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				match keys::handle_key(key.code, keybindings, &mut events) {
					// TODO: use sth more elegant than bool return type
					Ok(false) => return Ok(()),
					Err(e) => return Err(e),
					_ => {}
				};
			}
		}
		if last_tick.elapsed() >= tick_rate {
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
