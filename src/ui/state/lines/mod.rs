mod line;

pub use line::Line;

use crate::config::Styles;
use crate::config::{Fields, TableFormatter};
use anyhow::Result;
use itertools::izip;
use ratatui::{
    prelude::{Backend, Constraint},
    style::Style,
    widgets::{Cell, Row, Table, TableState},
    Frame,
};
use std::cmp::max;

pub struct Lines {
    pub lines: Vec<Line>,
    pub selected: Vec<bool>,
    pub styles: Styles,
    pub fields: Fields,
    pub index_after_header_lines: usize,
    pub cursor_index: Option<usize>,
    // TODO: deprecate in future
    pub table_state: TableState,
}

impl Lines {
    pub fn new(fields: Fields, styles: Styles, header_lines: usize) -> Self {
        Self {
            lines: vec![],
            selected: vec![],
            fields,
            cursor_index: None,
            styles,
            index_after_header_lines: header_lines,
            table_state: TableState::default(),
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // TODO: do as much as possible in update_lines to improve performance
        let rows: Vec<Row> = izip!(&self.lines, &self.selected)
            .map(|(line, &selected)| {
                // TODO: consider replacing Vec<bool> with Vec<Style> directly
                let selected_style = if selected {
                    self.styles.selected
                } else {
                    self.styles.line
                };

                Row::new(vec![Cell::from(" ").style(selected_style), line.draw()])
            })
            .collect();

        let table = Table::new(rows)
            .widths(&[Constraint::Length(1), Constraint::Percentage(100)])
            .column_spacing(0);

        frame.render_stateful_widget(table, frame.size(), &mut self.table_state);
    }

    // TODO: might be better suited as a new() method or similar
    pub fn update_lines(&mut self, lines: String) -> Result<()> {
        let formatted: Vec<Option<String>> = match lines.as_str().format_as_table(&self.fields)? {
            Some(formatted) => formatted.lines().map(str::to_owned).map(Some).collect(),
            None => vec![None; lines.lines().count()],
        };

        self.lines = izip!(lines.lines(), formatted)
            .enumerate()
            .map(|(i, (unformatted, formatted))| {
                let style = if i < self.index_after_header_lines {
                    self.styles.header
                } else {
                    self.styles.line
                };

                Line::new(unformatted.to_owned(), formatted, style)
            })
            .collect();

        self.selected.resize(self.lines.len(), false);
        self.calibrate_cursor();

        Ok(())
    }

    // Moving cursor

    // TODO: don't use isize, instead use an enum Up|Down and saturating_{add,sub}
    fn move_cursor(&mut self, index: isize) {
        let old = self.get_cursor_position();
        let new = if self.lines.is_empty() {
            None
        } else {
            let first = self.index_after_header_lines as isize;
            let last = self.last_index() as isize;
            Some(index.clamp(first, last) as usize)
        };

        self.cursor_index = new;
        self.table_state.select(self.cursor_index);
        self.adjust_cursor_style(old, new);
    }

    fn get_cursor_position(&self) -> Option<usize> {
        self.cursor_index
    }

    fn calibrate_cursor(&mut self) {
        match self.get_cursor_position() {
            None => self.move_cursor_to_first_line(),
            Some(i) => self.move_cursor(i as isize),
        };
    }

    pub fn move_cursor_down(&mut self, steps: usize) {
        if let Some(i) = self.get_cursor_position() {
            self.move_cursor(i as isize + steps as isize);
        }
    }

    pub fn move_cursor_up(&mut self, steps: usize) {
        if let Some(i) = self.get_cursor_position() {
            self.move_cursor(i as isize - steps as isize);
        }
    }

    pub fn move_cursor_to_first_line(&mut self) {
        self.move_cursor(self.index_after_header_lines as isize);
    }

    pub fn move_cursor_to_last_line(&mut self) {
        self.move_cursor(self.last_index() as isize);
    }

    // Styling cursor

    fn adjust_cursor_style(&mut self, old: Option<usize>, new: Option<usize>) {
        if let Some(old_index) = old {
            self.update_line_style(old_index, self.styles.line);
        }
        if let Some(new_index) = new {
            self.update_line_style(new_index, self.styles.cursor);
        }
    }

    pub fn update_line_style(&mut self, index: usize, new_style: Style) {
        if let Some(line) = self.lines.get_mut(index) {
            line.update_style(new_style);
        }
    }

    // Selecting lines

    pub fn select_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            if let Some(selected) = self.selected.get_mut(i) {
                *selected = true;
            }
        }
    }

    pub fn unselect_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            if let Some(selected) = self.selected.get_mut(i) {
                *selected = false;
            }
        }
    }

    pub fn toggle_selection_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            if let Some(selected) = self.selected.get_mut(i) {
                *selected = !(*selected);
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selected.fill(true);
    }

    pub fn unselect_all(&mut self) {
        self.selected.fill(false);
    }

    // Getting selected lines

    fn get_line_under_cursor(&self) -> Option<String> {
        self.get_cursor_position()
            .and_then(|i| self.get_unformatted(i))
    }

    // TODO: not pretty API, maybe make cursor_line and selected_lines distinct types
    pub fn get_selected_lines(&self) -> Option<(String, String)> {
        self.get_line_under_cursor().map(|cursor_line| {
            let selected_lines = if self.selected.contains(&true) {
                izip!(self.unformatted(), self.selected.iter())
                    .filter_map(|(line, &selected)| selected.then(|| line.to_owned()))
                    .collect::<Vec<String>>()
                    .join("\n")
            } else {
                cursor_line.clone()
            };
            (cursor_line, selected_lines)
        })
    }

    // Formatting

    pub fn unformatted(&self) -> Vec<&String> {
        self.lines.iter().map(Line::unformatted).collect()
    }

    pub fn get_unformatted(&self, index: usize) -> Option<String> {
        self.lines.get(index).map(|line| line.unformatted().clone())
    }

    // Miscellaneous

    fn last_index(&self) -> usize {
        if self.lines.is_empty() {
            self.index_after_header_lines
        } else {
            max(self.index_after_header_lines, self.lines.len() - 1)
        }
    }
}
