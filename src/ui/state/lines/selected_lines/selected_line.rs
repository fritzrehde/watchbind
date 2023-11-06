use ratatui::{style::Style, widgets::Cell};

/// Stores whether a line is selected or not, and also stores the UI to draw.
#[derive(Clone)]
pub struct LineSelection {
    /// Whether a line is selected or not.
    line_selected: LineSelected,
    /// Widget containing the string that will be displayed in the TUI.
    /// The content string is a single space character.
    /// The content string is styled/colored if the line is selected.
    displayed: Cell<'static>,
}

/// Stores whether a line is selected or not.
#[derive(Clone)]
pub enum LineSelected {
    Unselected,
    Selected,
}

impl<'a> LineSelection {
    /// Create a new line selection.
    pub fn new(line_selected: LineSelected, style: Style) -> Self {
        Self {
            line_selected,
            displayed: Self::draw_unstyled().style(style),
        }
    }

    /// Draw line selection as a styled ratatui widget.
    pub fn draw(&self) -> Cell {
        self.displayed.clone()
    }

    /// Draw a new line selection as an unstyled ratatui widget.
    fn draw_unstyled() -> Cell<'a> {
        Cell::from(" ")
    }

    /// Select the line.
    pub fn select(&mut self, selected_style: Style) {
        self.line_selected = LineSelected::Selected;
        // TODO: remove clone
        self.displayed = self.displayed.clone().style(selected_style);
    }

    /// Unselect the line.
    pub fn unselect(&mut self, unselected_style: Style) {
        self.line_selected = LineSelected::Unselected;
        // TODO: remove clone
        self.displayed = self.displayed.clone().style(unselected_style);
    }

    /// Toggle the selection of the line.
    pub fn toggle_selection(&mut self, selected_style: Style, unselected_style: Style) {
        match self.line_selected {
            LineSelected::Unselected => self.select(selected_style),
            LineSelected::Selected => self.unselect(unselected_style),
        }
    }

    /// Return whether the line is selected.
    pub fn is_selected(&self) -> bool {
        matches!(self.line_selected, LineSelected::Selected)
    }
}
