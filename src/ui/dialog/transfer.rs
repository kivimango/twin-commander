use crate::core::calculate_progress_percentage;
use fs_extra::{
    dir::TransitProcess as DirTransitProcess, file::TransitProcess as FileTransitProcess,
};
use humansize::{SizeFormatter, DECIMAL};
use std::{
    io::Stdout,
    path::Path,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::Instant,
};
use std::{path::PathBuf, sync::mpsc};
use termion::{event::Key, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Abstraction of file transfers (copy/move) for reusing
/// the same TransferDialog fo every different file transfers.
pub trait TransferStrategy {
    fn transfer_dir<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<TransferProgress>,
    );
    fn transfer_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<TransferProgress>,
    );
}

// Convenient type for sending two different type of data through a channel:
// dont need two distinct (tx,rx)
pub enum TransferProgress {
    DirTransfer(DirTransitProcess),
    FileTransfer(FileTransitProcess),
    None,
}

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

enum TransferDialogStatus {
    WaitingForConfirmation,
    Transfering,
    TransferFinished,
}

impl Default for TransferDialogStatus {
    fn default() -> Self {
        TransferDialogStatus::WaitingForConfirmation
    }
}

pub struct TransferDialog<T> {
    copy_progress: TransferProgress,
    focused_button: Buttons,
    source: PathBuf,
    destination: PathBuf,
    status: TransferDialogStatus,
    strategy: T,
    rx: Option<Receiver<TransferProgress>>,
    should_quit: bool,
    start_time: Instant,
}

impl<T> TransferDialog<T>
where
    T: TransferStrategy,
{
    pub(crate) fn new<P: AsRef<Path>>(source: P, destination: P, transfer_model: T) -> Self {
        TransferDialog {
            copy_progress: TransferProgress::None,
            focused_button: Buttons::Ok,
            source: PathBuf::from(source.as_ref()),
            destination: PathBuf::from(destination.as_ref()),
            status: TransferDialogStatus::default(),
            strategy: transfer_model,
            rx: None,
            should_quit: false,
            start_time: Instant::now(),
        }
    }

    pub(crate) fn handle_key(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {
                if let TransferDialogStatus::WaitingForConfirmation = self.status {
                    match self.focused_button {
                        Buttons::Ok => {
                            self.start_time = Instant::now();
                            self.status = TransferDialogStatus::Transfering;
                            let (tx, rx) = mpsc::channel();
                            self.rx = Some(rx);
                            if self.source.is_dir() {
                                self.strategy.transfer_dir::<&std::path::Path>(
                                    self.source.as_ref(),
                                    self.destination.as_ref(),
                                    tx,
                                );
                            } else if self.source.is_file() {
                                self.strategy.transfer_file::<&std::path::Path>(
                                    self.source.as_ref(),
                                    self.destination.as_ref(),
                                    tx,
                                );
                            }
                        }
                        Buttons::Cancel => self.should_quit = true,
                    }
                }
            }
            Key::Left | Key::Right | Key::Up | Key::Down => {
                self.focused_button.next();
            }
            _ => {}
        }
    }

    pub(crate) fn tick(&mut self) {
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(copy_progress) => {
                    self.copy_progress = copy_progress;
                }
                Err(_err) => match _err {
                    TryRecvError::Disconnected => {
                        self.status = TransferDialogStatus::TransferFinished;
                        self.should_quit = true;
                    }
                    TryRecvError::Empty => {}
                },
            }
        }
    }

    pub(crate) fn render(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        match self.status {
            TransferDialogStatus::WaitingForConfirmation => {
                self.show_confirmation_dialog(frame, area)
            }
            TransferDialogStatus::Transfering => self.show_transfer_progress(frame, area),
            TransferDialogStatus::TransferFinished => (),
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

    fn show_transfer_progress(
        &self,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
        area: Rect,
    ) {
        let (total_percent, partial_percent) = match &self.copy_progress {
            TransferProgress::DirTransfer(dir_progress) => (
                calculate_progress_percentage(dir_progress.copied_bytes, dir_progress.total_bytes),
                calculate_progress_percentage(
                    dir_progress.file_bytes_copied,
                    dir_progress.file_total_bytes,
                ),
            ),
            TransferProgress::FileTransfer(file_progress) => (
                0,
                calculate_progress_percentage(
                    file_progress.copied_bytes,
                    file_progress.total_bytes,
                ),
            ),
            TransferProgress::None => (0, 0),
        };
        let file_name = match &self.copy_progress {
            TransferProgress::DirTransfer(dir_progress) => dir_progress.file_name.clone(),
            TransferProgress::FileTransfer(_) => self.source.display().to_string(),
            TransferProgress::None => String::new(),
        };
        let (copied_bytes, total_bytes) = match &self.copy_progress {
            TransferProgress::DirTransfer(dir_progress) => {
                (dir_progress.file_bytes_copied, dir_progress.total_bytes)
            }
            TransferProgress::FileTransfer(file_progress) => {
                (file_progress.copied_bytes, file_progress.total_bytes)
            }
            TransferProgress::None => (0, 0),
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
            .margin(1)
            .split(area);

        let block = Block::default()
            .border_type(tui::widgets::BorderType::Rounded)
            .borders(Borders::ALL);

        let current_file_label = Paragraph::new(Span::styled(
            format!("Current: {}", file_name),
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
                SizeFormatter::new(copied_bytes, DECIMAL),
                SizeFormatter::new(total_bytes, DECIMAL)
            ),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Left);

        let secs = self.start_time.elapsed().as_secs() % 60;
        let mins = (self.start_time.elapsed().as_secs() / 60) % 60;
        let hours = (self.start_time.elapsed().as_secs() / 60) / 60;
        let label_total_time = Paragraph::new(Span::styled(
            format!("{}h:{}m:{}s", hours, mins, secs),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Center);

        let label_filesizes = Paragraph::new(Span::styled(
            format!(
                "{}/{}",
                SizeFormatter::new(copied_bytes, DECIMAL),
                SizeFormatter::new(total_bytes, DECIMAL)
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

    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }
}
