use humansize::{SizeFormatter, DECIMAL};
use std::path::Path;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Color, Style},
    tui::{
        layout::{Constraint, Direction, Layout, Rect},
        style::Stylize,
        text::{Line, Span, Text},
        widgets::{Block, Gauge, Paragraph, Wrap},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State, StateValue,
};

use super::DialogMessage;
use crate::{app::ApplicationMessage, worker_event::WorkerEvent};

pub const TAG_SOURCE: &str = "source";
pub const TAG_TARGET: &str = "target";

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

/// A base UI component for displaying confirmation dialog for file moving operations.
/// It displays the path of the source directory, the path of the target directory and two buttons,
/// to accept (Ok) or cancel the operation.
/// It is used both for Copy and Move dialog.
///
/// # Example 1
/// ```rust
/// let copy_dialog = TransferConfirmationDialog::default()
///     .source("/")
///     .target("/root/")
///     .keep_source(true)
///     .title("Confirm copy");
/// ```
///
/// # Example 2
/// ```rust
/// let move_dialog = TransferConfirmationDialog::default()
///     .source("/home/brad/.config/")
///     .target("/home/rebecca/")
///     .keep_source(false)
///     .title("Confirm move");
/// ```

pub struct TransferConfirmationDialog {
    focused_button: Buttons,
    keep_source: bool,
    properties: Props,
}

impl TransferConfirmationDialog {
    /// Sets the flag indicating that the operation should preserve the source file(s) after completing,
    /// i.e: differentiating between move and copy operations.
    pub fn keep_source(mut self, keep: bool) -> Self {
        self.keep_source = keep;
        self
    }
    /// Sets the source path to be displayed.
    pub fn source<P: AsRef<Path>>(mut self, source: P) -> Self {
        let source = source.as_ref().to_string_lossy().to_string();
        self.properties
            .set(Attribute::Custom(TAG_SOURCE), AttrValue::String(source));
        self
    }

    /// Sets the target path to be displayed.
    pub fn target<P: AsRef<Path>>(mut self, target: P) -> Self {
        let target = target.as_ref().to_string_lossy().to_string();
        self.properties
            .set(Attribute::Custom(TAG_TARGET), AttrValue::String(target));
        self
    }

    /// Sets the title of the dialog to be displayed in the top center of the dialog's border.
    pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
        let title = title.as_ref().to_string();
        self.properties.set(
            Attribute::Title,
            AttrValue::Title((title, Alignment::Center)),
        );
        self
    }
}

impl Default for TransferConfirmationDialog {
    fn default() -> Self {
        let mut properties = Props::default();
        properties.set(Attribute::Background, AttrValue::Color(Color::White));
        properties.set(Attribute::Foreground, AttrValue::Color(Color::Black));
        properties.set(Attribute::HighlightedColor, AttrValue::Color(Color::Cyan));
        properties.set(Attribute::Color, AttrValue::Color(Color::Gray));

        TransferConfirmationDialog {
            focused_button: Buttons::Cancel,
            keep_source: true,
            properties: Props::default(),
        }
    }
}

impl MockComponent for TransferConfirmationDialog {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value)
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.properties.get(attr)
    }

    fn state(&self) -> State {
        State::One(StateValue::Bool(self.keep_source))
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let button_titles = {
            match self.focused_button {
                Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
                Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
            }
        };
        let layout = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        let title = self
            .properties
            .get_or(
                Attribute::Title,
                AttrValue::Title((String::from("Copy"), Alignment::Center)),
            )
            .unwrap_title();
        let source = self
            .properties
            .get(Attribute::Custom(TAG_SOURCE))
            .unwrap()
            .unwrap_string();
        let target = self
            .properties
            .get(Attribute::Custom(TAG_TARGET))
            .unwrap()
            .unwrap_string();

        let background = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(Color::White))
            .unwrap_color();
        let text_color = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Black))
            .unwrap_color();
        let title_color = self
            .properties
            .get_or(Attribute::HighlightedColor, AttrValue::Color(Color::Cyan))
            .unwrap_color();
        let input_text_color = self
            .properties
            .get_or(Attribute::Color, AttrValue::Color(Color::Gray))
            .unwrap_color();

        let button_styles = {
            match self.focused_button {
                Buttons::Ok => (
                    Style::default().bg(Color::Cyan).fg(Color::White),
                    Style::default().fg(text_color),
                ),
                Buttons::Cancel => (
                    Style::default().fg(Color::Black),
                    Style::default().bg(Color::Cyan).fg(Color::White),
                ),
            }
        };

        let block = Block::default()
            .bg(Color::White)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().bg(background).fg(text_color))
            .borders(tuirealm::tui::widgets::Borders::ALL)
            .title_style(
                Style::default()
                    .bg(background)
                    .fg(title_color)
                    .add_modifier(tuirealm::tui::prelude::Modifier::BOLD),
            )
            .title_top(title.0)
            .title_alignment(title.1);

        let spans = vec![
            Line::from(vec![Span::styled(
                "Source:",
                Style::default().fg(text_color),
            )]),
            Line::from(vec![Span::styled(
                source,
                Style::default().bg(title_color).fg(input_text_color),
            )]),
            Line::from(vec![Span::styled(
                "Destination:",
                Style::default().fg(text_color),
            )]),
            Line::from(vec![Span::styled(
                target,
                Style::default().bg(title_color).fg(input_text_color),
            )]),
        ];

        let text = Text::from(spans);
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);
        let buttons = Paragraph::new(Line::from(vec![
            Span::styled(button_titles.0, button_styles.0),
            Span::styled(button_titles.1, button_styles.1),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
        frame.render_widget(buttons, layout[4])
    }
}

impl Component<ApplicationMessage, WorkerEvent> for TransferConfirmationDialog {
    fn on(&mut self, event: Event<WorkerEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. })
            | Event::Keyboard(KeyEvent {
                code: Key::Function(5),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog)),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. })
            | Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            })
            | Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.focused_button.next();
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.focused_button {
                Buttons::Ok => {
                    return Some(ApplicationMessage::Dialog(DialogMessage::BeginTransfer(
                        self.keep_source,
                    )))
                }
                Buttons::Cancel => {
                    return Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog))
                }
            },
            _ => {}
        }
        None
    }
}

pub struct TransferProgressDialog {
    keep_source: bool,
    properties: Props,
}

impl TransferProgressDialog {
    pub fn new() -> Self {
        let mut properties = Props::default();
        properties.set(Attribute::Background, AttrValue::Color(Color::White));
        properties.set(Attribute::Foreground, AttrValue::Color(Color::Black));
        properties.set(Attribute::HighlightedColor, AttrValue::Color(Color::Cyan));
        properties.set(Attribute::Color, AttrValue::Color(Color::Gray));

        TransferProgressDialog {
            keep_source: true,
            properties,
        }
    }

    /// Sets the flag indicating that the operation should preserve the source file(s) after completing,
    /// i.e: differentiating between move and copy operations.
    pub fn keep_source(mut self, keep: bool) -> Self {
        self.keep_source = keep;
        self
    }
    /// Sets the source path to be displayed.
    pub fn source<P: AsRef<Path>>(mut self, source: P) -> Self {
        let source = source.as_ref().to_string_lossy().to_string();
        self.properties
            .set(Attribute::Custom(TAG_SOURCE), AttrValue::String(source));
        self
    }

    /// Sets the target path to be displayed.
    pub fn target<P: AsRef<Path>>(mut self, target: P) -> Self {
        let target = target.as_ref().to_string_lossy().to_string();
        self.properties
            .set(Attribute::Custom(TAG_TARGET), AttrValue::String(target));
        self
    }

    /// Sets the title of the dialog to be displayed in the top center of the dialog's border.
    pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
        let title = title.as_ref().to_string();
        self.properties.set(
            Attribute::Title,
            AttrValue::Title((title, Alignment::Center)),
        );
        self
    }
}

impl MockComponent for TransferProgressDialog {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value)
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.properties.get(attr)
    }

    fn state(&self) -> State {
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        /*let (total_percent, partial_percent) = match &self.copy_progress {
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
        };*/
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

        let title = self
            .properties
            .get_or(
                Attribute::Title,
                AttrValue::Title((String::from(""), Alignment::Center)),
            )
            .unwrap_title();
        let source = self
            .properties
            .get(Attribute::Custom(TAG_SOURCE))
            .unwrap()
            .unwrap_string();
        let target = self
            .properties
            .get(Attribute::Custom(TAG_TARGET))
            .unwrap()
            .unwrap_string();
        let current_filename = self
            .properties
            .get_or(Attribute::Content, AttrValue::String(String::from("N/A")))
            .unwrap_string();
        let progress_percent = self
            .properties
            .get(Attribute::Value)
            .unwrap()
            .unwrap_payload()
            .unwrap_tup2();
        let (total_percent, partial_percent) = (
            progress_percent.0.unwrap_u16(),
            progress_percent.1.unwrap_u16(),
        );
        let progress_in_bytes = self
            .properties
            .get_or(Attribute::Width, AttrValue::String(String::from("N/A")))
            .unwrap_string();

        let background = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(Color::White))
            .unwrap_color();
        let text_color = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Black))
            .unwrap_color();
        let title_color = self
            .properties
            .get_or(Attribute::HighlightedColor, AttrValue::Color(Color::Cyan))
            .unwrap_color();
        let input_text_color = self
            .properties
            .get_or(Attribute::Color, AttrValue::Color(Color::Gray))
            .unwrap_color();

        let block = Block::default()
            .bg(Color::White)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().bg(background).fg(text_color))
            .borders(tuirealm::tui::widgets::Borders::ALL)
            .title_style(
                Style::default()
                    .bg(background)
                    .fg(title_color)
                    .add_modifier(tuirealm::tui::prelude::Modifier::BOLD),
            )
            .title_top(title.0)
            .title_alignment(title.1);

        let path_spans = vec![Line::from(vec![
            Span::styled(
                format!("Current: {}", current_filename),
                Style::default().fg(text_color),
            ),
            Span::styled(format!("To: {}", target), Style::default().fg(text_color)),
        ])];

        let progress_total = Gauge::default()
            .percent(total_percent)
            .gauge_style(Style::default().fg(Color::LightBlue));
        let progress_partial = Gauge::default()
            .percent(partial_percent)
            .gauge_style(Style::default().fg(Color::LightBlue));
        let label_remaining_size =
            Paragraph::new(Span::styled(progress_in_bytes, text_color)).alignment(Alignment::Left);

        /*let secs = self.start_time.elapsed().as_secs() % 60;
        let mins = (self.start_time.elapsed().as_secs() / 60) % 60;
        let hours = (self.start_time.elapsed().as_secs() / 60) / 60;
        let label_total_time = Paragraph::new(Span::styled(
            format!("{}h:{}m:{}s", hours, mins, secs),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Center);*/

        let label_filesizes = Paragraph::new(Span::styled(
            format!(
                "{}/{}",
                SizeFormatter::new(0u16, DECIMAL), //copied_bytes
                SizeFormatter::new(0u16, DECIMAL)  //total_bytes
            ),
            Style::default().fg(Color::White),
        ))
        .alignment(Alignment::Right);

        let pause_button = Span::styled("[ ] Pause ", Style::default().fg(Color::White));
        let cancel_button = Span::styled("[ ] Cancel ", Style::default().fg(Color::White));
        let background_button = Span::styled("[ ] Background", Style::default().fg(Color::White));
        let buttons = Paragraph::new(Text::from(Line::from(vec![
            pause_button,
            cancel_button,
            background_button,
        ])))
        .alignment(Alignment::Center);

        frame.render_widget(block, dialog_area);
        //frame.render_widget(current_file_label, layout[0]);
        //frame.render_widget(dest_label, layout[1]);
        frame.render_widget(progress_total, layout[2]);
        frame.render_widget(progress_partial, layout[3]);
        frame.render_widget(label_remaining_size, layout[4]);
        //frame.render_widget(label_total_time, layout[4]);
        frame.render_widget(label_filesizes, layout[4]);
        frame.render_widget(buttons, layout[5]);
    }
}

impl Component<ApplicationMessage, WorkerEvent> for TransferProgressDialog {
    fn on(&mut self, event: Event<WorkerEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog))
            }
            _ => None,
        }
    }
}
