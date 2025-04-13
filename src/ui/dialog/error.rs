use super::DialogMessage;
use crate::{app::ApplicationMessage, worker_event::WorkerEvent};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::{Alignment, Borders, Color, Style},
    tui::{
        layout::Rect,
        text::{Line, Span, Text},
        widgets::{block::Title, Block, Paragraph, Wrap},
        Frame,
    },
    AttrValue, Attribute, Component, Event, MockComponent, Props, State,
};

const DEFAULT_BACKGROUND: Color = Color::Red;
const DEFAULT_TEXT_COLOR: Color = Color::White;

/// A generic dialog used to display error messages.
/// The dialog can be closed by pressing the Esc, F10 or the Enter keys.
pub struct ErrorDialog {
    title: String,
    error_message: String,
    button_title: String,
    properties: Props,
}

impl ErrorDialog {
    pub fn new() -> Self {
        ErrorDialog {
            title: String::new(),
            error_message: String::new(),
            button_title: String::new(),
            properties: Props::default(),
        }
    }

    /// Sets the title of the dialog
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the error message to be displayed
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.error_message = message.into();
        self
    }

    /// Sets the confirmation button's title
    pub fn with_button_title<S: Into<String>>(mut self, button_title: S) -> Self {
        self.button_title = button_title.into();
        self
    }
}

impl MockComponent for ErrorDialog {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value);
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
        let border = self
            .properties
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let background_color = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(DEFAULT_BACKGROUND))
            .unwrap_color();
        let text_color = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(DEFAULT_TEXT_COLOR))
            .unwrap_color();

        let spans = vec![
            Line::from(vec![Span::styled(
                &self.error_message,
                Style::default().fg(text_color),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                &self.button_title,
                Style::default().bg(Color::LightBlue).fg(text_color),
            )]),
        ];
        let text = Text::from(spans);

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(Title::from(self.title.clone()).alignment(Alignment::Center))
                    .borders(border.sides)
                    .style(Style::default().bg(background_color).fg(text_color)),
            )
            .centered()
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(background_color).fg(text_color));

        frame.render_widget(paragraph, area);
    }
}

impl Component<ApplicationMessage, WorkerEvent> for ErrorDialog {
    fn on(&mut self, event: Event<WorkerEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Function(10),
                ..
            }) => Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog)),
            _ => None,
        }
    }
}
