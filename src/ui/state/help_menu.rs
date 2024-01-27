use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::KeybindingsPrintable;

use super::EnvVariables;

pub struct HelpMenu {
    env_variables: Arc<Mutex<EnvVariables>>,
    /// A local/non-shared copy of the shared `env_variables`.
    env_variables_copy: EnvVariables,
    keybindings: KeybindingsPrintable,
    vertical_scroll_index: usize,
    vertical_scroll_state: ScrollbarState,
}

// TODO: scrollbar should be hidden if not necessary; currently it's always shown

impl HelpMenu {
    pub fn new(keybindings: KeybindingsPrintable, env_variables: Arc<Mutex<EnvVariables>>) -> Self {
        HelpMenu {
            env_variables,
            env_variables_copy: EnvVariables::default(),
            keybindings,
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll_index: 0,
            // vertical_scroll_state: ScrollbarState::default()
            //     .content_length(keybindings_str.lines().count() as u16),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // TODO: maybe in the future, when we add more features for manipulating ENV variable state, we have to fetch the new
        let popup_area = centered_rect(90, 90, frame.size());
        // Get the inner popup width, so take borders into account.
        let popup_width = popup_area.width - 2;

        let rendered_text = format!(
            "ENV VARIABLES:\n{}\nKEYBINDINGS:\n{}\n",
            self.env_variables_copy.display(popup_width),
            self.keybindings.display(popup_width)
        );

        let text: Text = rendered_text.into();

        // Render the paragraph with the updated scroll state
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("help").borders(Borders::ALL))
            .alignment(Alignment::Left)
            // scroll offset for each axis: (y, x)
            .scroll((self.vertical_scroll_index as u16, 0));

        // Render the scrollbar next to the paragraph
        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);

        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            popup_area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );
    }

    fn update_vertical_scroll_index(&mut self, index: usize) {
        self.vertical_scroll_index = index;
        self.vertical_scroll_state = self.vertical_scroll_state.position(index);
    }

    // Moving

    pub fn move_down(&mut self, steps: usize) {
        // TODO: The lines might be wrapped, so we might actually have more indexes than, and therefore don't know what the last index is
        // TODO: Ideally, we only need to scroll if help content doesn't fit onto screen. But we don't know what fits on the screen currently, because we don't know if text got wrapped to the next line
        self.update_vertical_scroll_index(self.vertical_scroll_index.saturating_add(steps));
    }

    pub fn move_up(&mut self, steps: usize) {
        self.update_vertical_scroll_index(self.vertical_scroll_index.saturating_sub(steps));
    }

    pub fn move_to_first(&mut self) {
        // TODO: Since we don't allow last here, for the sake of consistency we don't allow first either for now
    }

    pub fn move_to_last(&mut self) {
        // TODO: The lines might be wrapped, so we might actually have more indexes than, and therefore don't know what the last index is
    }

    // Showing and hiding

    // TODO: this assumption is INCORRECT!!! Either update this all the time or make sure the assumption holds

    /// Update the internal string representation of the ENV variables. Our
    /// current assumption is that once we are in the help menu, we cannot
    /// manipulate the ENV variables until we return to the normal state. That
    /// is why we only update the state here, and not everytime the help menu
    /// is rendered.
    pub async fn show(&mut self) {
        // TODO: here, we would also have to set the vertical scroll length
        let env_variables = self.env_variables.lock().await;
        self.env_variables_copy = env_variables.clone();
    }

    pub fn hide(&mut self) {
        self.update_vertical_scroll_index(0);
    }
}

/// Helper function to create a centered rect using up certain percentage
/// of the available rect `r`.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
