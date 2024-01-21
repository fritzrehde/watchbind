mod env_variables;
mod help_menu;
mod lines;

use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use ratatui::Frame;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::{Fields, OperationExecutable, Operations, OperationsParsed, Styles};

use self::{
    help_menu::HelpMenu,
    lines::{CursorLine, Lines, SelectedLines},
};

pub use self::env_variables::{EnvVariable, EnvVariables};

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

    pub fn draw(&mut self, frame: &mut Frame) {
        self.lines.render(frame);

        if let Mode::HelpMenu = self.mode {
            self.help_menu.render(frame);
        }
    }
}

// TODO: replace with std::LazyLock once stable
static CURSOR_LINE_ENV_VAR: Lazy<EnvVariable> =
    Lazy::new(|| "line".parse().expect("should be valid env var"));
static SELECTED_LINES_ENV_VAR: Lazy<EnvVariable> =
    Lazy::new(|| "lines".parse().expect("should be valid env var"));

// API for Lines
impl State {
    /// Set both the cursor line as well as the selected lines in the UI as
    /// global environment variables for all future processes.
    pub async fn add_cursor_and_selected_lines_to_env(&mut self) {
        // TODO: get_selected_lines is sync and computationally intensive, maybe use spawn_blocking
        if let Some((cursor_line, selected_lines)) = self.get_cursor_line_and_selected_lines() {
            let new_env_variables: EnvVariables = [
                ((*CURSOR_LINE_ENV_VAR).clone(), cursor_line.into()),
                ((*SELECTED_LINES_ENV_VAR).clone(), selected_lines.into()),
            ]
            .into_iter()
            .collect();
            self.set_envs(new_env_variables).await;
        };
    }

    /// Unset the env variables for the cursor line and selected lines.
    pub async fn remove_cursor_and_selected_lines_from_env(&mut self) {
        self.unset_env(&CURSOR_LINE_ENV_VAR).await;
        self.unset_env(&SELECTED_LINES_ENV_VAR).await;
    }

    pub fn update_lines(&mut self, new_lines: String) -> Result<()> {
        self.lines.update_lines(new_lines)
    }

    pub fn get_cursor_line_and_selected_lines(&mut self) -> Option<(CursorLine, SelectedLines)> {
        self.lines.get_cursor_line_and_selected_lines()
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

    // TODO: make the "cursor moving" a trait/use rust type state pattern; this is a performance bottleneck, since we always have to match the current mode/state; ideally, we just transition to a state, and then never call any matches until we transition to the next state; the hard part is that we don't have distinct states, since they both still need each other in render all

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

    /// Generate initial environment variables. The commands to set each
    /// environment variable are blockingly executed.
    pub async fn generate_initial_env_vars(
        &mut self,
        initial_env_ops_parsed: OperationsParsed,
    ) -> Result<()> {
        let initial_env_ops =
            Operations::from_parsed(initial_env_ops_parsed.clone(), &self.get_env());

        // TODO: consider trying to use async iterators to do this in one iterator pass (instead of the mut hashmap) once stable
        for (i, op) in initial_env_ops.into_iter().enumerate() {
            match (i, op.executable) {
                (_, OperationExecutable::SetEnv(env_var, blocking_cmd)) => {
                    let cmd_output = blocking_cmd.execute().await?;
                    self.set_env(env_var, cmd_output).await;
                }
                (op_index, _) => {
                    // Delay retrieval of printable `OperationParsed` until
                    // this error case => improve performance for correct use
                    // of "set-env".
                    let other_op = initial_env_ops_parsed
                        .into_iter()
                        .nth(op_index)
                        .expect("length of `Operations` and `OperationsParsed` should be same");
                    bail!("Only `set-env` operations allowed during initial environment variable generation, but received invalid operation: {}", other_op);
                }
            }
        }
        Ok(())
    }

    pub fn get_env(&self) -> Arc<Mutex<EnvVariables>> {
        self.env_variables.clone()
    }

    /// Set environment variable `env_var` to `value`.
    pub async fn set_env(&mut self, env_var: EnvVariable, value: String) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.set_env(env_var, value);
    }

    pub async fn set_envs(&mut self, new_env_variables: EnvVariables) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.merge_new_envs(new_env_variables);
    }

    /// Unset an environment variable.
    pub async fn unset_env(&mut self, env_var: &EnvVariable) {
        let mut env_variables = self.env_variables.lock().await;
        env_variables.unset_env(env_var)
    }

    /// Unset multiple environment variables.
    pub async fn unset_envs(&mut self, env_vars: &[EnvVariable]) {
        let mut env_variables = self.env_variables.lock().await;
        for env in env_vars {
            env_variables.unset_env(env);
        }
    }

    pub async fn read_into_env(&mut self, _env: &EnvVariable) {
        todo!()
    }
}
