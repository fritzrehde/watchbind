use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::io::{stdout, Stdout};

pub type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub struct TerminalManager {
    pub terminal: Terminal,
}

impl TerminalManager {
    pub fn new() -> Result<Self> {
        let terminal = Self::create_tui()?;
        let mut terminal_manager = TerminalManager { terminal };
        terminal_manager.show_tui()?;

        Ok(terminal_manager)
    }

    // TODO: maybe we don't have to create the stdout, backend variables again, only once at creation. Optimize that later
    fn create_tui() -> Result<Terminal> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        Ok(terminal)
    }

    pub fn show_tui(&mut self) -> Result<()> {
        self.terminal = Self::create_tui()?;

        Ok(())
    }

    pub fn hide_tui(&mut self) -> Result<()> {
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

impl Drop for TerminalManager {
    fn drop(&mut self) {
        // TODO: remove unwrap
        self.hide_tui().unwrap();
    }
}
