use anyhow::Result;
use crossterm::{
	event::{DisableMouseCapture, EnableMouseCapture},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Stdout};
use tui::backend::CrosstermBackend;

pub type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub struct TerminalManager {
	pub terminal: Terminal,
}

impl TerminalManager {
	pub fn new() -> Result<TerminalManager> {
		enable_raw_mode()?;
		let mut stdout = stdout();
		execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
		let backend = CrosstermBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;
		terminal.hide_cursor()?;

		Ok(TerminalManager { terminal })
	}

	pub fn restore(&mut self) -> Result<()> {
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
