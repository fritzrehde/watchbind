use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::{backend::CrosstermBackend, Frame};
use std::io::{stdout, Stdout};

type Backend = CrosstermBackend<Stdout>;
type Terminal = ratatui::Terminal<Backend>;

/// A Terminal User Interface (TUI).
pub struct Tui {
    // TODO: don't make public (means moving the UI::draw! macro here)
    pub terminal: Terminal,
}

impl Tui {
    /// Initialise and show a new TUI.
    pub fn new() -> Result<Self> {
        let terminal = Self::create_new_terminal()?;
        let mut terminal_manager = Tui { terminal };

        terminal_manager.show()?;

        Ok(terminal_manager)
    }

    /// Create a new terminal that will write the TUI to stdout.
    fn create_new_terminal() -> Result<Terminal> {
        Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
    }

    /// Draw provided frame to TUI.
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// Show the TUI.
    fn show(&mut self) -> Result<()> {
        enable_raw_mode()?;
        crossterm::execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;

        Ok(())
    }

    /// Show a TUI which has been painted over by another TUI program.
    pub fn restore(&mut self) -> Result<()> {
        // Our own TUI was painted over by another program previously, so we
        // must create a new one.
        self.terminal = Self::create_new_terminal()?;

        self.show()?;

        Ok(())
    }

    /// Hide the TUI, but don't unpaint the TUI we have already drawn. This is
    /// useful if another program wants to paint a TUI onto the screen, and we
    /// don't want to temporarily return to the user's terminal for a
    /// split-second.
    pub fn hide(&mut self) -> Result<()> {
        disable_raw_mode()?;

        // The trick to not unpainting our TUI is to not leave the alternate
        // screen like we would normally do when hiding the TUI.

        self.terminal.show_cursor()?;

        Ok(())
    }

    /// Exit the TUI, which entails completely unpainting the TUI, and returning
    /// to the user's terminal.
    fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        crossterm::execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = self.exit() {
            log::error!("Tearing down the TUI failed with: {}", e);
        }
    }
}
