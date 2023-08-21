use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Margin, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub struct HelpMenu {
    is_shown: bool,
    body: String,
    vertical_scroll_index: usize,
    vertical_scroll_state: ScrollbarState,
}

// TODO: scrollbar should be hidden if not necessary; currently it's always shown

impl HelpMenu {
    pub fn new(help_menu_body: String) -> Self {
        HelpMenu {
            vertical_scroll_state: ScrollbarState::default()
                .content_length(help_menu_body.lines().count() as u16),
            is_shown: false,
            body: help_menu_body,
            vertical_scroll_index: 0,
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {
        if self.is_shown {
            let area = centered_rect(50, 50, frame.size());

            let text: Text = self.body.as_str().into();

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

    pub fn show(&mut self) {
        self.is_shown = true;
    }

    pub fn hide(&mut self) {
        self.is_shown = false;
        self.update_vertical_scroll_index(0);
    }

    pub fn is_shown(&self) -> bool {
        self.is_shown
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
