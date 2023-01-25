use std::{io::Stdout, path::PathBuf};
use termion::{event::Key, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
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
        let dialog_area = Rect::new(area.x, area.y, area.width, area.height);
        let layout = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(tui::layout::Direction::Vertical)
            .margin(1)
            .split(area);

        let block = Block::default()
            .border_type(tui::widgets::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("Copy file(s)")
            .title_alignment(Alignment::Center);
        let label_src = Paragraph::new(Text::styled("Source:", Style::default().fg(Color::White)));
        let label_src_path = Paragraph::new(Text::styled(
            self.source.display().to_string(),
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        let label_dest = Paragraph::new(Text::styled(
            "Destination:",
            Style::default().fg(Color::White),
        ));
        let label_dest_path = Paragraph::new(Text::styled(
            self.destination.display().to_string(),
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        let buttons = Paragraph::new(Spans::from(vec![
            Span::styled(button_titles.0, Style::default().fg(Color::White)),
            Span::styled(button_titles.1, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(block, dialog_area);
        frame.render_widget(label_src, layout[0]);
        frame.render_widget(label_src_path, layout[1]);
        frame.render_widget(label_dest, layout[2]);
        frame.render_widget(label_dest_path, layout[3]);
        frame.render_widget(buttons, layout[4]);
    }
}
