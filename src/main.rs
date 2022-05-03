use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
	io,
	time::{Duration, Instant},
};
use tui::{
	backend::{Backend, CrosstermBackend},
	style::{Color, Style},
	widgets::{List, ListItem},
	Frame, Terminal,
};
use events::Events;

mod cli;
mod events;
mod command;

fn main() -> Result<(), io::Error> {
	// parse args and options
	let args = cli::parse_args();
	let command = args.value_of("command").expect("Command is required by clap");

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// run tui program
	let tick_rate = Duration::from_millis(250);
	let mut events = Events::new(command::output_lines(command));
	run(&mut terminal, &mut events, tick_rate)?;

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	Ok(())
}

fn run<B: Backend>(
	terminal: &mut Terminal<B>,
	events: &mut Events,
	tick_rate: Duration,
) -> io::Result<()> {
	let mut last_tick = Instant::now();
	loop {
		terminal.draw(|f| ui(f, events))?;

		let timeout = tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::from_secs(0));
		if crossterm::event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				match key.code {
					KeyCode::Char('q') => return Ok(()),
					KeyCode::Left => events.unselect(),
					KeyCode::Down => events.next(),
					KeyCode::Up => events.previous(),
					KeyCode::Char('j') => events.next(),
					KeyCode::Char('k') => events.previous(),
					KeyCode::Char('g') => events.first(),
					KeyCode::Char('G') => events.last(),
					_ => {}
				}
			}
		}
		if last_tick.elapsed() >= tick_rate {
			last_tick = Instant::now();
		}
	}
}

fn ui<B: Backend>(f: &mut Frame<B>, events: &mut Events) {
	let items: Vec<ListItem> = events
		.items.iter()
		.map(|i| ListItem::new(i.as_ref()))
		.collect();
	let list = List::new(items)
		.style(Style::default().fg(Color::White))
		.highlight_style(Style::default().fg(Color::Black).bg(Color::White));
	f.render_stateful_widget(list, f.size(), &mut events.state);
}
