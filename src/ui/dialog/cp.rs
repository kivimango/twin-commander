use fs_extra::dir::{copy_with_progress, CopyOptions, TransitProcess, TransitState};
use humansize::{SizeFormatter, DECIMAL};
use std::{
    io::Stdout,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Instant,
};
use termion::{event::Key, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::core::calculate_progress_percentage;

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

enum CopyDialogStatus {
    WaitingForConfirmation,
    Copying,
    CopyFinished,
}

impl Default for CopyDialogStatus {
    fn default() -> Self {
        CopyDialogStatus::WaitingForConfirmation
    }
}

pub struct CopyDialog {
    copy_progress: TransitProcess,
    focused_button: Buttons,
    source: PathBuf,
    destination: PathBuf,
    status: CopyDialogStatus,
    tx: Sender<TransitProcess>,
    rx: Receiver<TransitProcess>,
    start_time: Instant,
}

impl CopyDialog {
    pub(crate) fn new(source: PathBuf, destination: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        CopyDialog {
            copy_progress: TransitProcess {
                copied_bytes: 0,
                total_bytes: 0,
                file_bytes_copied: 0,
                file_total_bytes: 0,
                file_name: String::new(),
                state: TransitState::Normal,
            },
            focused_button: Buttons::Ok,
            source,
            destination,
            status: CopyDialogStatus::default(),
            tx,
            rx,
            start_time: Instant::now(),
        }
    }

    pub(crate) fn handle_key(&mut self, key: Key) {
        match key {
            Key::Char('\n') => match self.status {
                CopyDialogStatus::WaitingForConfirmation => {
                    self.start_time = Instant::now();
                    self.status = CopyDialogStatus::Copying;
                    copy_dir(
                        self.source.clone(),
                        self.destination.clone(),
                        self.tx.clone(),
                    );
                }
                _ => {}
            },
            Key::Left | Key::Right | Key::Up | Key::Down => {
                self.focused_button.next();
            }
            _ => {}
        }
    }

    pub(crate) fn tick(&mut self) {
        match self.rx.try_recv() {
            Ok(copy_progress) => {
                self.copy_progress = copy_progress;
            }
            Err(_err) => match _err {
                TryRecvError::Disconnected => {
                    self.status = CopyDialogStatus::CopyFinished;
                }
                TryRecvError::Empty => {}
            },
        }
    }

    pub fn render(&self, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>, area: Rect) {
        match self.status {
            CopyDialogStatus::WaitingForConfirmation => self.show_confirmation_dialog(frame, area),
            CopyDialogStatus::Copying => self.show_copy_progress(frame, area),
            CopyDialogStatus::CopyFinished => (),
        }
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

    fn show_copy_progress(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        let (total_percent, partial_percent) = calculate_progress_percentage(&self.copy_progress);
        let dialog_area = Rect::new(area.x, area.y, area.width, area.height);
        let layout = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);

        let block = Block::default()
            .border_type(tui::widgets::BorderType::Rounded)
            .borders(Borders::ALL);

        let current_file_label = Paragraph::new(Span::styled(
            format!("Current: {}", self.copy_progress.file_name),
            Style::default().fg(Color::White),
        ));
        let dest_filename = self.destination.display().to_string();
        let dest_label = Paragraph::new(Span::styled(
            format!("To: {}", dest_filename),
            Style::default().fg(Color::White),
        ));

        let progress_total = Gauge::default()
            .percent(total_percent as u16)
            .gauge_style(Style::default().fg(Color::LightBlue));
        let progress_partial = Gauge::default()
            .percent(partial_percent as u16)
            .gauge_style(Style::default().fg(Color::LightBlue));
        let label_remaining_size = Paragraph::new(Span::styled(
            format!(
                "{}/{}",
                SizeFormatter::new(self.copy_progress.copied_bytes, DECIMAL),
                SizeFormatter::new(self.copy_progress.total_bytes, DECIMAL)
            ),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Left);

        let secs = self.start_time.elapsed().as_secs() % 60;
        let mins = (self.start_time.elapsed().as_secs() / 60) % 60;
        let hours = (self.start_time.elapsed().as_secs() / 60) / 60;
        let label_total_time = Paragraph::new(Span::styled(
            format!("{}:{}:{}", hours, mins, secs),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Center);

        let label_filesizes = Paragraph::new(Span::styled(
            format!(
                "{}/{}",
                SizeFormatter::new(self.copy_progress.file_bytes_copied, DECIMAL),
                SizeFormatter::new(self.copy_progress.file_total_bytes, DECIMAL)
            ),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Right);

        let pause_button = Span::styled("[ ] Pause ", Style::default().fg(Color::White));
        let cancel_button = Span::styled("[ ] Cancel ", Style::default().fg(Color::White));
        let background_button = Span::styled("[ ] Background", Style::default().fg(Color::White));
        let buttons = Paragraph::new(Text::from(Spans::from(vec![
            pause_button,
            cancel_button,
            background_button,
        ])))
        .alignment(Alignment::Center);

        frame.render_widget(block, dialog_area);
        frame.render_widget(current_file_label, layout[0]);
        frame.render_widget(dest_label, layout[1]);
        frame.render_widget(progress_total, layout[2]);
        frame.render_widget(progress_partial, layout[3]);
        frame.render_widget(label_remaining_size, layout[4]);
        frame.render_widget(label_total_time, layout[4]);
        frame.render_widget(label_filesizes, layout[4]);
        frame.render_widget(buttons, layout[5]);
    }
}

fn copy_dir(from: PathBuf, to: PathBuf, tx: Sender<TransitProcess>) {
    let mut options = CopyOptions::new();
    options.buffer_size = 8 * 1024 * 1024; // TODO: configurable buffer, default is 1MB
    let from_1 = from.clone();
    let to_1 = to.clone();

    thread::spawn(move || {
        let progress_handler = |progress_info: TransitProcess| {
            if let Ok(_) = tx.send(progress_info) {}
            fs_extra::dir::TransitProcessResult::Abort
        };
        let _result = copy_with_progress(
            AsRef::<Path>::as_ref(&from_1),
            AsRef::<Path>::as_ref(&to_1),
            &options,
            progress_handler,
        );
    });
}
