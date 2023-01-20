use std::{io::Stdout, path::PathBuf};
use termion::{event::Key, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
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

impl Default for DeleteDialogState {
    fn default() -> Self {
        DeleteDialogState::WaitingForConfirmation
    }
}

pub struct RmDirDialog {
    dialog_state: DeleteDialogState,
    focused_button: FocusedButton,
    path: Vec<PathBuf>,
}

impl RmDirDialog {
    pub fn new(path: Vec<PathBuf>) -> Self {
        RmDirDialog {
            dialog_state: DeleteDialogState::default(),
            focused_button: FocusedButton::Cancel,
            path,
        }
    }

    pub fn handle_keys(&mut self, key: Key) {
        match key {
            Key::Char('\n') => match self.dialog_state {
                DeleteDialogState::WaitingForConfirmation => match self.focused_button {
                    FocusedButton::Ok => self.dialog_state = DeleteDialogState::Deleting,
                    FocusedButton::Cancel => {}
                },
                _ => {}
            },
            Key::Char('\t') | Key::Right | Key::Left | Key::Up | Key::Down => {
                self.focused_button.next()
            }
            _ => (),
        }
    }

    pub fn render(&self, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>, area: Rect) {
        match self.dialog_state {
            DeleteDialogState::WaitingForConfirmation => self.show_confirmation_dialog(frame, area),
            DeleteDialogState::Deleting => self.show_deleting_files(frame, area),
            DeleteDialogState::Deleted => {}
        }
    }

    fn show_confirmation_dialog(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        let button_titles = {
            match self.focused_button {
                FocusedButton::Ok => ("[X] OK ", "[ ] Cancel"),
                FocusedButton::Cancel => ("[ ] OK ", "[X] Cancel"),
            }
        };
        let spans = vec![
            Spans::from(vec![Span::styled(
                self.confirm_msg(),
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::styled(
                self.get_name(),
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )]),
            Spans::from(vec![
                Span::styled(button_titles.0, Style::default().fg(Color::Black)),
                Span::styled(button_titles.1, Style::default().fg(Color::Black)),
            ]),
        ];
        let text = Text::from(spans);
        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Delete")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::LightRed).fg(Color::White)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::LightRed).fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn show_deleting_files(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        let spans = vec![
            Spans::from(vec![Span::styled(
                "Deleting:",
                Style::default().fg(Color::Black),
            )]),
            Spans::from(vec![Span::styled(
                self.get_name(),
                Style::default().fg(Color::Black),
            )]),
        ];
        let text = Text::from(spans);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
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
