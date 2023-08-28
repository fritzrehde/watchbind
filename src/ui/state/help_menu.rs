use std::sync::Arc;

use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use tokio::sync::Mutex;

use super::EnvVariables;

pub struct HelpMenu {
    env_variables: Arc<Mutex<EnvVariables>>,
    keybindings_str: String,
    env_variables_str: String,
    vertical_scroll_index: usize,
    vertical_scroll_state: ScrollbarState,
}

// TODO: scrollbar should be hidden if not necessary; currently it's always shown
// TODO: we should display a mapping of all set EnvVariables (with their actual values)

impl HelpMenu {
    pub fn new(keybindings_str: String, env_variables: Arc<Mutex<EnvVariables>>) -> Self {
        HelpMenu {
            // vertical_scroll_state: ScrollbarState::default()
            //     .content_length(keybindings_str.lines().count() as u16),
            env_variables,
            keybindings_str,
            env_variables_str: String::default(),
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll_index: 0,
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // TODO: maybe in the future, when we add more features for manipulating ENV variable state, we have to fetch the new
        let area = centered_rect(50, 50, frame.size());

        let rendered_text = format!(
            "ENV VARIABLES:\n{}\nKEYBINDINGS:\n{}\n",
            self.env_variables_str, self.keybindings_str
        );

        // let text: Text = self.keybindings_str.as_str().into();
        let text: Text = rendered_text.into();

        // Render the paragraph with the updated scroll state
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("help").borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            // scroll offset for each axis: (y, x)
            .scroll((self.vertical_scroll_index as u16, 0));

        // Render the scrollbar next to the paragraph
        frame.render_widget(paragraph, area);

        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );
    }

    fn update_vertical_scroll_index(&mut self, index: usize) {
        self.vertical_scroll_index = index;
        self.vertical_scroll_state = self.vertical_scroll_state.position(index as u16);
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
        self.env_variables_str = env_variables.to_string();
    }

    pub fn hide(&mut self) {
        self.update_vertical_scroll_index(0);
    }
}

/// Helper function to create a centered rect using up certain percentage
/// of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
