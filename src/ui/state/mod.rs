mod env_variables;
mod help_menu;
mod lines;

use self::{help_menu::HelpMenu, lines::Lines};
use crate::config::{Fields, Styles};
use anyhow::Result;
use ratatui::{
    backend::Backend,
    prelude::{Alignment, Constraint, Direction, Layout},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use env_variables::{EnvVariable, EnvVariables};

pub struct State {
    mode: Mode,
    lines: Lines,
    help_menu: HelpMenu,
    pub env_variables: Arc<Mutex<EnvVariables>>,
}

#[derive(Default)]
enum Mode {
    #[default]
    Normal,
    HelpMenu,
}

impl State {
    pub fn new(
        header_lines: usize,
        fields: Fields,
        styles: Styles,
        keybindings_str: String,
        env_variables: EnvVariables,
    ) -> Self {
        let env_variables = Arc::new(Mutex::new(env_variables));
        Self {
            mode: Mode::default(),
            lines: Lines::new(fields, styles, header_lines),
            help_menu: HelpMenu::new(keybindings_str, env_variables.clone()),
            env_variables,
        }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // TODO: only draw text input widget if currently in use
        // TODO: don't limit the text input line height (usually 1) to 1, but let it grow to accomodate wrapping text input

        // Split UI vertically into the lines (top) and the text-input (bottom)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
            .split(frame.size());

        let lines_area = chunks[0];
        let text_input_area = chunks[1];

        // Render each component individually
        self.lines.render(frame, lines_area);

        // TODO: move rendering text input to own file
        let text_input: Text = ":text input".into();
        let paragraph = Paragraph::new(text_input)
            .block(Block::new().borders(Borders::NONE))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, text_input_area);

        if let Mode::HelpMenu = self.mode {
            self.help_menu.render(frame);
        }
    }

    // API for Lines

    pub fn update_lines(&mut self, new_lines: String) -> Result<()> {
        self.lines.update_lines(new_lines)
    }

    fn get_cursor_line_and_selected_lines(&mut self) -> Option<(String, String)> {
        self.lines.get_selected_lines()
    }

    /// Set both the cursor line as well as the selected lines in the UI as
    /// global environment variables for all future processes.
    pub async fn add_lines_to_env(&mut self) -> Result<()> {
        // TODO: get_selected_lines is sync and computationally intensive, maybe use spawn_blocking
        if let Some((cursor_line, selected_lines)) = self.get_cursor_line_and_selected_lines() {
            let new_env_variables: EnvVariables = [
                ("line".parse()?, cursor_line),
                ("lines".parse()?, selected_lines),
            ]
            .into_iter()
            .collect();
            self.set_env_vars(new_env_variables).await;
        };
        Ok(())
    }

    pub fn select(&mut self) {
        self.lines.select_current();
    }

    pub fn unselect(&mut self) {
        self.lines.unselect_current();
    }

    pub fn toggle_selection(&mut self) {
        self.lines.toggle_selection_current();
    }

    pub fn select_all(&mut self) {
        self.lines.select_all();
    }

    pub fn unselect_all(&mut self) {
        self.lines.unselect_all();
    }

    // API for Help Menu

    pub async fn show_help_menu(&mut self) {
        self.help_menu.show().await;
        self.mode = Mode::HelpMenu;
    }

    pub fn hide_help_menu(&mut self) {
        self.help_menu.hide();
        self.mode = Mode::Normal;
    }

    pub async fn toggle_help_menu(&mut self) {
        match self.mode {
            Mode::Normal => self.show_help_menu().await,
            Mode::HelpMenu => self.hide_help_menu(),
        }
    }

    // API for both Lines and Help Menu

    // TODO: make the "cursor moving" a trait; this is a performance bottleneck, since we always have to match the current mode/state; ideally, we just transition to a state, and then never call any matches until we transition to the next state; the hard part is that we don't have distinct states, since they both still need each other in render all

    pub fn move_down(&mut self, steps: usize) {
        match self.mode {
            Mode::Normal => self.lines.move_cursor_down(steps),
            Mode::HelpMenu => self.help_menu.move_down(steps),
        }
    }

    pub fn move_up(&mut self, steps: usize) {
        match self.mode {
            Mode::Normal => self.lines.move_cursor_up(steps),
            Mode::HelpMenu => self.help_menu.move_up(steps),
        }
    }

    pub fn move_to_first(&mut self) {
        match self.mode {
            Mode::Normal => self.lines.move_cursor_to_first_line(),
            Mode::HelpMenu => self.help_menu.move_to_first(),
        }
    }

    pub fn move_to_last(&mut self) {
        match self.mode {
            Mode::Normal => self.lines.move_cursor_to_last_line(),
            Mode::HelpMenu => self.help_menu.move_to_last(),
        }
    }

    // API for environment variables

    pub fn get_env(&self) -> Arc<Mutex<EnvVariables>> {
        self.env_variables.clone()
    }

    /// Merge new env variables with existing env variables.
    pub async fn set_env_vars(&mut self, new_env_variables: EnvVariables) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.merge_new_env_variables(new_env_variables);
    }

    /// Unset an env variable.
    pub async fn unset_env_var(&mut self, env_var: &EnvVariable) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.unset_env_variable(env_var)
    }

    /// Sets an env variable to a string value.
    pub async fn set_env_var(&mut self, env_var: &EnvVariable, value: String) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.add_env_variable(env_var.to_owned(), value);
    }

    /// Initiate reading user input into an env var by creating a UI text field.
    /// This will return immediately instead of waiting for submission by the
    /// user. Therefore, this does not add any new env variables yet.
    pub async fn request_read_into_env(&self, env_var: &EnvVariable) {
        let value = todo!();
    }
}
