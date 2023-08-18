mod line;

pub use line::Line;

use crate::config::Styles;
use anyhow::Result;
use derive_more::IntoIterator;
use itertools::izip;
use ratatui::{
    prelude::{Backend, Constraint},
    style::Style,
    widgets::{Cell, Row, Table, TableState},
    Frame,
};
use std::{cmp::max, io::Write};
use tabwriter::TabWriter;

#[derive(IntoIterator)]
pub struct Lines {
    // TODO: remove, unclear what this does
    #[into_iterator(ref)]
    pub lines: Vec<Line>,
    pub selected: Vec<bool>,
    pub field_separator: Option<String>,
    pub styles: Styles,
    // TODO: rename to first_index_after_header where cursor can be placed
    pub header_lines: usize,
    pub cursor_index: Option<usize>,

    // TODO: deprecate in future
    pub table_state: TableState,
}

impl Lines {
    pub fn new(field_separator: Option<String>, styles: Styles, header_lines: usize) -> Self {
        Self {
            lines: vec![],
            selected: vec![],
            field_separator,
            cursor_index: None,
            styles,
            header_lines,
            table_state: TableState::default(),
        }
    }

    pub fn update(&mut self, lines: String) -> Result<()> {
        let formatted: Vec<Option<String>> = match &self.field_separator {
            Some(separator) => {
                // TODO: cleaner syntax
                let mut tw = TabWriter::new(vec![]);
                write!(&mut tw, "{}", lines.replace(separator, "\t"))?;
                tw.flush()?;

                String::from_utf8(tw.into_inner()?)?
                    .lines()
                    .map(|line| Some(line.to_owned()))
                    .collect()
            }
            None => lines.lines().map(|_| None).collect(),
        };

        self.lines = izip!(lines.lines(), formatted)
            .enumerate()
            .map(|(i, (unformatted, formatted))| {
                let style = if i < self.header_lines {
                    self.styles.header
                } else {
                    self.styles.line
                };

                Line::new(unformatted.to_owned(), formatted, style)
            })
            .collect();

        self.selected.resize(self.lines.len(), false);
        self.cursor_calibrate();

        Ok(())
    }

    fn cursor_position(&mut self) -> Option<usize> {
        self.cursor_index
    }

    // TODO: don't use isize, instead use an enum Up|Down and saturating_{add,sub}
    fn cursor_move(&mut self, index: isize) {
        let old = self.cursor_position();
        let new = if self.lines.is_empty() {
            None
        } else {
            let first = self.header_lines as isize;
            let last = self.last_index() as isize;
            Some(index.clamp(first, last) as usize)
        };

        self.cursor_index = new;
        self.table_state.select(self.cursor_index);
        self.cursor_adjust_style(old, new);
    }

    fn cursor_calibrate(&mut self) {
        match self.cursor_position() {
            None => self.move_cursor_to_first_line(),
            Some(i) => self.cursor_move(i as isize),
        };
    }

    fn cursor_adjust_style(&mut self, old: Option<usize>, new: Option<usize>) {
        if let Some(old_index) = old {
            self.update_style(old_index, self.styles.line);
        }
        if let Some(new_index) = new {
            self.update_style(new_index, self.styles.cursor);
        }
    }

    fn get_cursor_line(&mut self) -> Option<String> {
        if let Some(i) = self.cursor_position() {
            self.get_unformatted(i)
        } else {
            None
        }
    }

    fn last_index(&self) -> usize {
        if self.lines.is_empty() {
            self.header_lines
        } else {
            max(self.header_lines, self.lines.len() - 1)
        }
    }

    pub fn update_style(&mut self, index: usize, new_style: Style) {
        if let Some(line) = self.lines.get_mut(index) {
            line.update_style(new_style);
        }
    }

    pub fn unformatted(&self) -> Vec<&String> {
        self.lines.iter().map(Line::unformatted).collect()
    }

    pub fn get_unformatted(&self, index: usize) -> Option<String> {
        self.lines.get(index).map(|line| line.unformatted().clone())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get_selected_lines(&mut self) -> Option<String> {
        if self.selected.contains(&true) {
            let lines: String = izip!(self.unformatted(), self.selected.iter())
                .filter_map(|(line, &selected)| selected.then(|| line.to_owned()))
                .collect::<Vec<String>>()
                .join("\n");
            Some(lines)
        } else {
            self.get_cursor_line()
        }
    }

    pub fn move_cursor_down(&mut self, steps: usize) {
        if let Some(i) = self.cursor_position() {
            self.cursor_move(i as isize + steps as isize);
        }
    }

    pub fn move_cursor_up(&mut self, steps: usize) {
        if let Some(i) = self.cursor_position() {
            self.cursor_move(i as isize - steps as isize);
        }
    }

    pub fn move_cursor_to_first_line(&mut self) {
        self.cursor_move(self.header_lines as isize);
    }

    pub fn move_cursor_to_last_line(&mut self) {
        self.cursor_move(self.last_index() as isize);
    }

    pub fn select_current(&mut self) {
        if let Some(i) = self.cursor_position() {
            if let Some(selected) = self.selected.get_mut(i) {
                *selected = true;
            }
        }
    }

    pub fn unselect_current(&mut self) {
        if let Some(i) = self.cursor_position() {
            if let Some(selected) = self.selected.get_mut(i) {
                *selected = false;
            }
        }
    }

    pub fn toggle_selection_current(&mut self) {
        if let Some(i) = self.cursor_position() {
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
}
