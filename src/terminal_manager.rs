use crossterm::{
	event::{DisableMouseCapture, EnableMouseCapture},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Error};
use tui::{backend::CrosstermBackend, Terminal};

pub struct TerminalManager {
	pub terminal: tui::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl TerminalManager {
	pub fn new() -> Result<TerminalManager, Error> {
		// setup terminal
		enable_raw_mode()?;
		let mut stdout = io::stdout();
		execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
		let backend = CrosstermBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;
		terminal.hide_cursor()?;

		Ok(TerminalManager { terminal })
	}

	pub fn restore(&mut self) -> Result<(), Error> {
		// restore terminal
		disable_raw_mode()?;
		execute!(
			self.terminal.backend_mut(),
			LeaveAlternateScreen,
			DisableMouseCapture
		)?;
		self.terminal.show_cursor()?;
		Ok(())
	}
}
