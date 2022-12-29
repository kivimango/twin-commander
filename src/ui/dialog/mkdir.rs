use std::{io::Error, path::PathBuf};
use termion::event::Key;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_input::{Input, InputRequest};

#[derive(PartialEq)]
/// Represents the state of the dialog
pub enum MkDirDialogState {
    WaitingForInput,
    DirCreated,
    DisplayErrorMessage(String),
}

/// Represents a dialog used for creating a new directory.
pub struct MkDirDialog {
    input: Input,
    parent_dir: PathBuf,
    state: MkDirDialogState,
}

impl MkDirDialog {
    pub fn new(parent_dir: PathBuf) -> Self {
        MkDirDialog {
            input: Input::default(),
            state: MkDirDialogState::WaitingForInput,
            parent_dir,
        }
    }

    pub fn create_dir(&mut self) -> Result<(), Error> {
        let mut parent_dir = self.parent_dir.clone();
        parent_dir.push(self.input.value());
        let result = std::fs::create_dir(parent_dir);
        match result {
            Ok(_) => {
                self.state = MkDirDialogState::DirCreated;
            }
            Err(ref error) => {
                self.state = MkDirDialogState::DisplayErrorMessage(error.to_string());
            }
        }
        result
    }

    pub fn handle_key(&mut self, key: Key) {
        match key {
            Key::Char(char) => {
                if char.is_ascii_alphabetic() {
                    self.input.handle(InputRequest::InsertChar(char));
                }
            }
            Key::Backspace => {
                self.input.handle(InputRequest::DeletePrevChar);
            }
            Key::Delete => {
                self.input.handle(InputRequest::DeleteNextChar);
            }
            _ => {}
        }
    }

    /// Returns a representation based on the actual state of the dialog to render.
    pub fn widget(&self) -> Paragraph {
        match &self.state {
            MkDirDialogState::WaitingForInput => self.display_input(),
            MkDirDialogState::DisplayErrorMessage(msg) => self.display_error(&msg),
            _ => self.display_input(),
        }
    }

    /// Returns the current state of the dialog
    pub fn state(&self) -> &MkDirDialogState {
        &self.state
    }

    fn display_input(&self) -> Paragraph {
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
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Creating a new directory",
                        Style::default().fg(Color::Cyan),
                    ))
                    .style(Style::default().fg(Color::Black).bg(Color::Gray))
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center),
            )
            .alignment(Alignment::Center)
    }

    fn display_error<'error_msg>(&self, error_message: &'error_msg str) -> Paragraph<'error_msg> {
        let spans = vec![
            Spans::from(vec![Span::styled(
                error_message,
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::styled(
                "[ OK ]",
                Style::default().fg(Color::White),
            )]),
        ];
        let text = Text::from(spans);
        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Error")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::LightRed).fg(Color::White)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::LightRed).fg(Color::Gray))
            .alignment(Alignment::Center)
    }
}
