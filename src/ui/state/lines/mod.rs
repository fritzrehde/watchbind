mod line;
mod selected_lines;

use self::selected_lines::LineSelections;
use crate::config::Styles;
use crate::config::{Fields, TableFormatter};
use anyhow::Result;
use derive_more::{From, Into};
use itertools::{izip, Itertools};
use ratatui::{
    prelude::Constraint,
    style::Style,
    widgets::{Row, Table, TableState},
    Frame,
};
use std::cmp::max;

pub use line::Line;

/// The state of the lines, which can be drawn in order to be displayed
/// in the UI.
pub struct Lines {
    /// Stores all output lines from the watched command.
    lines: Vec<Line>,
    /// Stores which lines are selected. The reason why this is separate
    /// from `lines` is that the `lines` are updated periodically,
    /// but the selection of lines stays persistant between `lines`-updates.
    /// However, the length of the `line_selections` will always be equal to
    /// that of `lines`, meaning it will be resized on `lines`-updates.
    line_selections: LineSelections,
    /// The styles used to style the `lines` and `line_selections`.
    styles: Styles,
    /// Specifies the delimiter and shown fields that should be displayed
    /// for each line.
    fields: Fields,
    /// The first index after the header lines, which is the smallest possible
    /// index the cursor can take.
    index_after_header_lines: usize,
    /// The line index of the cursor.
    cursor_index: Option<usize>,
    // TODO: deprecate in future
    table_state: TableState,
}

impl Lines {
    pub fn new(fields: Fields, styles: Styles, header_lines: usize) -> Self {
        Self {
            lines: vec![],
            line_selections: LineSelections::new(styles.selected, styles.non_cursor_non_header),
            fields,
            cursor_index: None,
            styles,
            index_after_header_lines: header_lines,
            table_state: TableState::default(),
        }
    }

    /// Render to frame.
    pub fn render(&mut self, frame: &mut Frame) {
        // TODO: do as much as possible in update_lines to improve performance
        let rows: Vec<Row> = izip!(self.lines.iter(), self.line_selections.iter())
            .map(|(line, selected)| Row::new(vec![selected.draw(), line.draw()]))
            .collect();

        let table = Table::new(rows)
            .widths(&[Constraint::Length(1), Constraint::Percentage(100)])
            .column_spacing(0);

        frame.render_stateful_widget(table, frame.size(), &mut self.table_state);
    }

    /// Update the lines to `new_lines`.
    pub fn update_lines(&mut self, new_lines: String) -> Result<()> {
        let formatted: Vec<Option<String>> =
            match new_lines.as_str().format_as_table(&self.fields)? {
                // All lines have formatting.
                Some(formatted) => formatted.lines().map(str::to_owned).map(Some).collect(),
                // No lines have formatting.
                None => vec![None; new_lines.lines().count()],
            };

        self.lines = izip!(new_lines.lines(), formatted)
            .enumerate()
            .map(|(i, (unformatted, formatted))| {
                let style = if i < self.index_after_header_lines {
                    self.styles.header
                } else {
                    self.styles.non_cursor_non_header
                };
                Line::new(unformatted.to_owned(), formatted, style)
            })
            .collect::<Result<_>>()?;

        // Resize the line selections to the same size as the lines.
        self.line_selections.resize(self.lines.len());

        self.calibrate_cursor();

        Ok(())
    }
}

// Moving cursor
impl Lines {
    // TODO: don't use isize, instead use an enum Up|Down and saturating_{add,sub}

    /// Move the cursor to `index`.
    fn move_cursor(&mut self, index: isize) {
        let old_cursor_index = self.get_cursor_position();
        let new_cursor_index = if self.lines.is_empty() {
            None
        } else {
            let first = self.index_after_header_lines as isize;
            let last = self.last_index() as isize;
            Some(index.clamp(first, last) as usize)
        };

        self.cursor_index = new_cursor_index;
        self.table_state.select(self.cursor_index);
        self.adjust_cursor_style(old_cursor_index, new_cursor_index);
    }

    /// Get the current cursor index, or `None` if there is currently no cursor.
    fn get_cursor_position(&self) -> Option<usize> {
        self.cursor_index
    }

    /// Calibrate the cursor. Calibration may be necessary if the cursor is
    /// still on a line that no longer exists.
    fn calibrate_cursor(&mut self) {
        match self.get_cursor_position() {
            None => self.move_cursor_to_first_line(),
            Some(i) => self.move_cursor(i as isize),
        };
    }

    /// Move the cursor down by `steps`.
    pub fn move_cursor_down(&mut self, steps: usize) {
        if let Some(i) = self.get_cursor_position() {
            self.move_cursor(i as isize + steps as isize);
        }
    }

    /// Move the cursor up by `steps`.
    pub fn move_cursor_up(&mut self, steps: usize) {
        if let Some(i) = self.get_cursor_position() {
            self.move_cursor(i as isize - steps as isize);
        }
    }

    /// Move the cursor to the first line.
    pub fn move_cursor_to_first_line(&mut self) {
        self.move_cursor(self.index_after_header_lines as isize);
    }

    /// Move the cursor to the last line.
    pub fn move_cursor_to_last_line(&mut self) {
        self.move_cursor(self.last_index() as isize);
    }
}

// Styling cursor
impl Lines {
    /// After changing cursor positions, the styles of the lines must be
    /// updated. Revert the style of the line the cursor was on previously
    /// (`old_cursor_index`), and update the line the cursor is on now
    /// (`new_cursor_index`). It is possible that there is no old or new
    /// cursor index.
    fn adjust_cursor_style(
        &mut self,
        old_cursor_index: Option<usize>,
        new_cursor_index: Option<usize>,
    ) {
        if let Some(old_index) = old_cursor_index {
            self.update_line_style(old_index, self.styles.non_cursor_non_header);
        }
        if let Some(new_index) = new_cursor_index {
            self.update_line_style(new_index, self.styles.cursor);
        }
    }

    /// Update the style of the line at `index`.
    pub fn update_line_style(&mut self, index: usize, new_style: Style) {
        if let Some(line) = self.lines.get_mut(index) {
            line.update_style(new_style);
        }
    }
}

// Selecting lines
impl Lines {
    /// Select the line that the cursor is currently on.
    pub fn select_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            self.line_selections.select_at_index(i);
        }
    }

    /// Unselect the line that the cursor is currently on.
    pub fn unselect_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            self.line_selections.unselect_at_index(i);
        }
    }

    /// Toggle the selection of the line that the cursor is currently on.
    pub fn toggle_selection_current(&mut self) {
        if let Some(i) = self.get_cursor_position() {
            self.line_selections.toggle_selection_at_index(i);
        }
    }

    /// Select all lines.
    pub fn select_all(&mut self) {
        self.line_selections.select_all();
    }

    /// Unselect all lines.
    pub fn unselect_all(&mut self) {
        self.line_selections.unselect_all();
    }
}

/// String content of the line on which the cursor is currently on.
#[derive(From, Into, Clone)]
pub struct CursorLine(String);

/// Concatenation of all contents of selected lines into string.
#[derive(From, Into)]
pub struct SelectedLines(String);

// Getting selected lines
impl Lines {
    /// Return the string content of the cursor line and the selected lines.
    /// If there are no selected lines, the cursor line is returned for both
    /// the cursor line and the selected lines.
    pub fn get_cursor_line_and_selected_lines(&self) -> Option<(CursorLine, SelectedLines)> {
        self.get_line_under_cursor().map(|cursor_line| {
            let mut selected_lines_iter = izip!(self.lines.iter(), self.line_selections.iter())
                .filter_map(|(line, selection)| {
                    selection.is_selected().then(|| line.unformatted_str())
                })
                .peekable();

            let selected_lines = match selected_lines_iter.peek() {
                // There are some selected lines.
                Some(_) => selected_lines_iter.join("\n"),
                // There are no selected lines.
                None => cursor_line.clone(),
            };

            (cursor_line.into(), selected_lines.into())
        })
    }

    /// Get the string content of the line that the cursor is currently on,
    /// or `None` if there is currently no cursor.
    fn get_line_under_cursor(&self) -> Option<String> {
        self.get_cursor_position()
            .and_then(|i| self.get_unformatted_line(i))
    }
}

// Miscellaneous
impl Lines {
    /// Get an owned, unformatted version of the line at `index`, or `None`
    /// if it doesn't exist.
    pub fn get_unformatted_line(&self, index: usize) -> Option<String> {
        self.lines.get(index).map(Line::unformatted_string)
    }

    /// Get the index of the last line. The returned index will never be within
    /// the header lines.
    fn last_index(&self) -> usize {
        if self.lines.is_empty() {
            self.index_after_header_lines
        } else {
            max(self.index_after_header_lines, self.lines.len() - 1)
        }
    }
}
