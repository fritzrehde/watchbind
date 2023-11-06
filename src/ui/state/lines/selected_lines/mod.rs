mod selected_line;

use derive_new::new;
use ratatui::style::Style;

pub use selected_line::LineSelection;

use self::selected_line::LineSelected;

/// All selected lines.
#[derive(new)]
pub struct LineSelections {
    #[new(default)]
    selections: Vec<LineSelection>,
    selected_style: Style,
    unselected_style: Style,
}

impl LineSelections {
    /// Get referencing iterator.
    pub fn iter(&self) -> impl Iterator<Item = &LineSelection> {
        self.selections.iter()
    }

    /// Resize the line selections to `new_len`.
    pub fn resize(&mut self, new_len: usize) {
        self.selections.resize(
            new_len,
            // If larger, extend vector with unselected lines.
            LineSelection::new(LineSelected::Unselected, self.unselected_style),
        )
    }

    /// Select all lines.
    pub fn select_all(&mut self) {
        self.selections.fill(LineSelection::new(
            LineSelected::Selected,
            self.selected_style,
        ));
    }

    /// Unselect all lines.
    pub fn unselect_all(&mut self) {
        self.selections.fill(LineSelection::new(
            LineSelected::Unselected,
            self.unselected_style,
        ));
    }

    /// Select the line that is at `index` in the vector.
    pub fn select_at_index(&mut self, index: usize) {
        if let Some(selection) = self.selections.get_mut(index) {
            selection.select(self.selected_style);
        }
    }

    /// Unselect the line that is at `index` in the vector.
    pub fn unselect_at_index(&mut self, index: usize) {
        if let Some(selection) = self.selections.get_mut(index) {
            selection.unselect(self.unselected_style);
        }
    }

    /// Toggle the selection of the line that is at `index` in the vector.
    pub fn toggle_selection_at_index(&mut self, index: usize) {
        if let Some(selection) = self.selections.get_mut(index) {
            selection.toggle_selection(self.selected_style, self.unselected_style);
        }
    }
}
