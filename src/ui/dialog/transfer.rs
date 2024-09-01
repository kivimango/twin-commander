use std::path::Path;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Color, Style},
    tui::{
        layout::{Constraint, Direction, Layout, Rect},
        style::Stylize,
        text::{Line, Span, Text},
        widgets::{Block, Paragraph, Wrap},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

use super::DialogMessage;
use crate::app::ApplicationMessage;

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
/// It displays a source directory, a target directory and two buttons, to accept (Ok) or cancel the operation.
struct TransferConfirmationDialog {
    focused_button: Buttons,
    properties: Props,
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
        State::None
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

/// UI component for displaying a confirmation dialog for file moving operations.
/// It is based on the TransferConfirmationDialog.
///
/// Key controls:
/// * F5: display/hide this dialog
/// * Tab | Left Arrow | Right Arrow: switch between the currently selected button
/// * Esc: Hides this dialog
/// * Enter: Activate currently selected button
#[derive(MockComponent)]
pub struct CopyConfirmationDialog {
    component: TransferConfirmationDialog,
}

impl Default for CopyConfirmationDialog {
    fn default() -> Self {
        let component = TransferConfirmationDialog::default();
        let dialog = CopyConfirmationDialog { component };
        dialog.title("Copy")
    }
}

impl CopyConfirmationDialog {
    /// Creates a new `CopyConfirmationDialog` with default properties.
    /// See `CopyConfirmationDialog::default()`.
    pub fn new() -> Self {
        CopyConfirmationDialog::default()
    }

    /// Sets the source path to be displayed.
    pub fn source<P: AsRef<Path>>(mut self, source: P) -> Self {
        let source = source.as_ref().to_string_lossy().to_string();
        self.component
            .properties
            .set(Attribute::Custom(TAG_SOURCE), AttrValue::String(source));
        self
    }

    /// Sets the target path to be displayed.
    pub fn target<P: AsRef<Path>>(mut self, target: P) -> Self {
        let target = target.as_ref().to_string_lossy().to_string();
        self.component
            .properties
            .set(Attribute::Custom(TAG_TARGET), AttrValue::String(target));
        self
    }

    /// Sets the title of the dialog to be displayed in the top center of the dialog's border.
    pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
        let title = title.as_ref().to_string();
        self.component.properties.set(
            Attribute::Title,
            AttrValue::Title((title, Alignment::Center)),
        );
        self
    }
}

impl Component<ApplicationMessage, NoUserEvent> for CopyConfirmationDialog {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
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
                self.component.focused_button.next();
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.focused_button {
                Buttons::Ok => {
                    return Some(ApplicationMessage::Dialog(DialogMessage::BeginTransfer(
                        true,
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
