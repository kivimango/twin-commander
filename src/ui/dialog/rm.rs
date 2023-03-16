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
    files: Vec<PathBuf>,
    focused_button: Buttons,
    should_quit: bool,
}

impl RmDirDialog {
    pub fn new(files: Vec<PathBuf>) -> Self {
        RmDirDialog {
            dialog_state: DeleteDialogState::default(),
            files,
            focused_button: Buttons::Cancel,
            should_quit: false,
        }
    }

    pub fn handle_keys(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {
                if let DeleteDialogState::WaitingForConfirmation = self.dialog_state {
                    match self.focused_button {
                        Buttons::Ok => {
                            self.dialog_state = DeleteDialogState::Deleting;
                            delete_files(&self.files);
                            self.dialog_state = DeleteDialogState::Deleted;
                            self.should_quit = true;
                        }
                        Buttons::Cancel => {
                            self.should_quit = true;
                        }
                    }
                }
            }
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

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn show_confirmation_dialog(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        let button_titles = {
            match self.focused_button {
                Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
                Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
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
                "Deleting file(s)",
                Style::default().fg(Color::Black),
            )]),
            /*Spans::from(vec![Span::styled(
                self.get_name(),
                Style::default().fg(Color::Black),
            )]),*/
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
        let count = self.files.len();
        if count == 1 {
            if let Some(file) = self.files.get(0) {
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
            format!("Are you sure you want to delete {} items ?", count)
        }
    }

    fn get_name(&self) -> String {
        let count = self.files.len();
        if count == 1 {
            if let Some(file) = self.files.get(0) {
                file.display().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

fn delete_files(files: &Vec<PathBuf>) {
    for file in files {
        if file.is_file() || file.is_symlink() {
            let _ = std::fs::remove_file(file);
        } else if file.is_dir() {
            let _ = std::fs::remove_dir_all(file);
        }
    }
}
