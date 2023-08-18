mod help_menu;
mod lines;

use self::{help_menu::HelpMenu, lines::Lines};
use crate::config::Styles;
use anyhow::Result;
use ratatui::{backend::Backend, Frame};

pub struct State {
    lines: Lines,
    help_menu: HelpMenu,
    mode: Mode,
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
        field_separator: Option<String>,
        styles: Styles,
        help_menu_body: String,
    ) -> Self {
        Self {
            lines: Lines::new(field_separator, styles.clone(), header_lines),
            help_menu: HelpMenu::new(help_menu_body),
            mode: Mode::default(),
        }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        self.lines.render(frame);
        self.help_menu.render(frame);
    }

    // API for Lines

    pub fn update_lines(&mut self, new_lines: String) -> Result<()> {
        self.lines.update_lines(new_lines)
    }

    pub fn get_selected_lines(&mut self) -> Option<String> {
        self.lines.get_selected_lines()
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

    pub fn show_help_menu(&mut self) {
        self.help_menu.show();
        self.mode = Mode::HelpMenu;
    }

    pub fn hide_help_menu(&mut self) {
        self.help_menu.hide();
        self.mode = Mode::Normal;
    }

    pub fn toggle_help_menu(&mut self) {
        if self.help_menu.is_shown() {
            self.hide_help_menu();
        } else {
            self.show_help_menu();
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
}
