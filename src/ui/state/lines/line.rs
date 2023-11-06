use ansi_to_tui::IntoText;
use anyhow::Result;
use itertools::Itertools;
use ratatui::{style::Style, text::Text, widgets::Cell};

pub struct Line {
    /// Unformatted string that has any ANSI escape codes stripped out.
    /// This string will be made available to the user's command's subshell
    /// through an environment variable.
    unformatted: String,
    /// The text string that contains the string content, is formatted
    /// according to the user's field separator, and is styled according to any
    /// ANSI codes. Does not contain the user style. Immutable for the lifetime
    /// of the line.
    displayed_text: Text<'static>,
    /// A cell containing the `displayed_text`, but with any user styles (style
    /// settings that should apply to the whole line), provided at creation
    /// and/or later, applied. If there is overlap in a setting between the
    /// `displayed_text`s style and the user style, the user style is
    /// prioritized.
    displayed: Cell<'static>,
}

impl<'a> Line {
    /// Create a new Line. Apply the `user_style` to the whole line.
    /// The formatted string was formatted according to the user's field
    /// separator.
    /// The unformatted and formatted strings may both contain ANSI escape
    /// codes, which will be converted incorporated into `displayed_text`.
    pub fn new(
        unformatted_ansi: String,
        formatted_ansi: Option<String>,
        user_style: Style,
    ) -> Result<Self> {
        let formatted_or_unformatted = formatted_ansi.as_ref().unwrap_or(&unformatted_ansi);

        let displayed_text = Self::format_line_content(formatted_or_unformatted).into_text()?;
        let displayed = Self::build_displayed_style(&displayed_text, user_style);

        let unformatted = unformatted_ansi.into_text()?.to_unformatted_string();

        Ok(Self {
            unformatted,
            displayed,
            displayed_text,
        })
    }

    /// Add one space before the line's content to create separation from the
    /// frame to the left.
    fn format_line_content(line_content: &str) -> String {
        format!(" {}", line_content)
    }

    /// Build the final style of the displayed cell, which consists of the
    /// displayed text's inherent style and the user style. If any style
    /// settings overlap, the user style is taken.
    fn build_displayed_style(displayed_text: &Text<'a>, user_style: Style) -> Cell<'a> {
        // We don't want to add the user style to the displayed text, so clone.
        let mut displayed_text = displayed_text.clone();
        // Merge the style from the displayed text and the user style, and
        // prioritise the user style.
        displayed_text.patch_style(user_style);
        // Also apply user style to whole cell, so areas outside the text but
        // still inside the cell are also styled.
        Cell::from(displayed_text).style(user_style)
    }

    /// Draw the line.
    pub fn draw(&self) -> Cell {
        self.displayed.clone()
    }

    /// Update the style of the whole line.
    pub fn update_style(&mut self, new_style: Style) {
        self.displayed = Self::build_displayed_style(&self.displayed_text, new_style);
    }

    /// Get the line as a &str.
    pub fn unformatted_str(&self) -> &str {
        &self.unformatted
    }

    /// Get the line as an owned String.
    pub fn unformatted_string(&self) -> String {
        self.unformatted.to_owned()
    }
}

trait ToUnformattedString {
    /// Extract the unformatted string underlying a `Text` object.
    fn to_unformatted_string(&self) -> String;
}

impl<'a> ToUnformattedString for Text<'a> {
    fn to_unformatted_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .join("\n")
    }
}
