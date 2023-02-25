use std::{
    io::{self},
    path::{Path, PathBuf},
};
use termion::event::Key;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_input::{Input, InputRequest};

enum Buttons {
    Ok,
    Cancel,
}

impl Buttons {
    fn next(&mut self) {
        match *self {
            Buttons::Ok => *self = Buttons::Cancel,
            Buttons::Cancel => *self = Buttons::Ok,
        }
    }
}

#[derive(PartialEq)]
/// Represents the state of the dialog
pub enum MkDirDialogState {
    WaitingForInput,
    DisplayErrorMessage(String),
}

/// Represents a dialog used for creating a new directory.
pub struct MkDirDialog {
    button: Buttons,
    input: Input,
    hide: bool,
    parent_dir: PathBuf,
    state: MkDirDialogState,
}

impl MkDirDialog {
    pub fn new<P>(parent_dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        MkDirDialog {
            button: Buttons::Ok,
            input: Input::default(),
            hide: false,
            state: MkDirDialogState::WaitingForInput,
            parent_dir: PathBuf::from(parent_dir.as_ref()),
        }
    }

    pub fn create_dir(&mut self) -> io::Result<()> {
        let mut parent_dir = self.parent_dir.clone();
        parent_dir.push(self.input.value());
        std::fs::create_dir(parent_dir)
    }

    pub fn handle_key(&mut self, key: Key) {
        match self.state {
            MkDirDialogState::WaitingForInput => match key {
                Key::Char('\n') => match self.button {
                    Buttons::Ok => match self.create_dir() {
                        Ok(_) => self.hide = true,
                        Err(error) => {
                            self.state = MkDirDialogState::DisplayErrorMessage(error.to_string())
                        }
                    },
                    Buttons::Cancel => self.hide = true,
                },
                Key::Char(char) => {
                    // TODO: regex for allowed chars in linux file names
                    if char.is_alphanumeric() {
                        self.input.handle(InputRequest::InsertChar(char));
                    }
                }
                Key::Backspace => {
                    self.input.handle(InputRequest::DeletePrevChar);
                }
                Key::Delete => {
                    self.input.handle(InputRequest::DeleteNextChar);
                }
                Key::Right | Key::Left | Key::Up | Key::Down => self.button.next(),
                _ => {}
            },
            MkDirDialogState::DisplayErrorMessage(_) => match key {
                Key::Char('\n') => self.state = MkDirDialogState::WaitingForInput,
                Key::Esc => self.hide = true,
                _ => {}
            },
        }
    }

    /// Returns a representation based on the actual state of the dialog to render.
    pub fn widget(&self) -> Paragraph {
        match &self.state {
            MkDirDialogState::WaitingForInput => self.display_input(),
            MkDirDialogState::DisplayErrorMessage(msg) => self.display_error(msg),
        }
    }

    /// Signals that the dialog should be closed or not
    pub fn should_hide(&self) -> bool {
        self.hide
    }

    fn display_input(&self) -> Paragraph {
        let button_titles = match self.button {
            Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
            Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
        };
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
                Span::styled(button_titles.0, Style::default().fg(Color::Black)),
                Span::styled(button_titles.1, Style::default().fg(Color::Black)),
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
