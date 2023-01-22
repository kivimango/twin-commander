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

pub struct CopyDialog {
    focused_button: Buttons,
    source: PathBuf,
    destination: PathBuf,
}

impl CopyDialog {
    pub(crate) fn new(source: PathBuf, destination: PathBuf) -> Self {
        CopyDialog {
            focused_button: Buttons::Ok,
            source,
            destination,
        }
    }

    pub(crate) fn handle_key(&mut self, key: Key) {
        match key {
            Key::Left | Key::Right | Key::Up | Key::Down => {
                self.focused_button.next();
            }
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>, area: Rect) {
        self.show_confirmation_dialog(frame, area)
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
                "Source:",
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::styled(
                self.source.display().to_string(),
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )]),
            Spans::from(vec![Span::styled(
                "Destination:",
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::styled(
                self.destination.display().to_string(),
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
                    .title("Copy file(s)")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::White).fg(Color::Black)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .alignment(Alignment::Left);
        frame.render_widget(p, area);
    }
}
