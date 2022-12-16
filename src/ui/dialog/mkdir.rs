use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph}, layout::Alignment,
};
use tui_input::Input;

/// Represents a dialog used for creating a new directory.
pub struct MkDirDialog {
    input: Input,
}

impl MkDirDialog {
    pub fn new() -> Self {
        MkDirDialog {
            input: Input::default(),
        }
    }
    
    /// Returns a representation of the dialog to render.
    pub fn widget(&self) -> Paragraph {
        let spans = vec![
            Spans::from(vec![Span::styled(
                "New directory name:",
                Style::default().fg(Color::Black),
            )]),
            Spans::from(Span::styled(
                self.input.value(),
                Style::default().bg(Color::Cyan).fg(Color::Black),
            )),
            Spans::from(vec![
                Span::styled("[ ] OK ", Style::default().fg(Color::Black)),
                Span::styled("[ ] Cancel", Style::default().fg(Color::Black)),
            ]),
        ];
        let text = Text::from(spans);
        Paragraph::new(text).block(
            Block::default()
                .title(Span::styled(
                    "Creating a new directory",
                    Style::default().fg(Color::Cyan),
                ))
                .style(Style::default().fg(Color::Black).bg(Color::Gray))
                .borders(Borders::ALL).title_alignment(Alignment::Center),
        ).alignment(Alignment::Center)
    }
}
