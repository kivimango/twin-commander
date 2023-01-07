use std::path::PathBuf;
use termion::event::Key;
use tui::{
    layout::Alignment,
    style::{Color, Style, Modifier},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

enum FocusedButton {
    Ok,
    Cancel,
}

impl FocusedButton {
    fn next(&mut self) {
        match *self {
            FocusedButton::Ok => *self = FocusedButton::Cancel,
            FocusedButton::Cancel => *self = FocusedButton::Ok,
        }
    }
}

pub enum DeleteDialogState {
    WaitingForConfirmation,
    Deleting,
    Deleted,
}

pub struct RmDirDialog {
    path: Vec<PathBuf>,
    focused_button: FocusedButton,
}

impl RmDirDialog {
    pub fn new(path: Vec<PathBuf>) -> Self {
        RmDirDialog {
            path,
            focused_button: FocusedButton::Cancel,
        }
    }

    pub fn handle_keys(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {}
            Key::Char('\t') | Key::Right | Key::Left | Key::Up | Key::Down => {
                self.focused_button.next()
            }
            _ => (),
        }
    }

    pub fn render(&self) -> Paragraph {
        let button_titles = {
            match self.focused_button {
                FocusedButton::Ok => ("[X] OK ", "[ ] Cancel"),
                FocusedButton::Cancel => ("[ ] OK ", "[X] Cancel"),
            }
        };
        let spans = vec![
            Spans::from(vec![
                Span::styled(self.confirm_msg(), Style::default().fg(Color::White)),
            ]),
            Spans::from(vec![
                Span::styled(self.get_name(), Style::default().fg(Color::Black).add_modifier(Modifier::BOLD))
            ]),
            Spans::from(vec![
                Span::styled(button_titles.0, Style::default().fg(Color::Black)),
                Span::styled(button_titles.1, Style::default().fg(Color::Black)),
            ]),
        ];
        let text = Text::from(spans);
        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Delete")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::LightRed).fg(Color::White)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::LightRed).fg(Color::Gray))
            .alignment(Alignment::Center)
    }

    /// Decides the confirmation message to be displayed to the user based on the type
    /// and the count of files on the path marked to delete.
    fn confirm_msg(&self) -> String {
        let count = self.path.len();
        if count == 1 {
            if let Some(file) = self.path.get(0) {
                if file.is_dir() {
                    String::from(
                        "Are you sure you want to delete this folder and all of its content ?",
                    )
                } else {
                    String::from("Are you sure you want to delete this file ?")
                }
            } else {
                String::from("Are you sure you want to delete this ?")
            }
        } else {
            String::from(format!("Are you sure you want to delete {} items ?", count))
        }
    }

    fn get_name(&self) -> String {
        let count = self.path.len();
        if count == 1 {
            if let Some(file) = self.path.get(0) {
                file.display().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}
