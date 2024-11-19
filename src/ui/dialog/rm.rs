use std::path::PathBuf;

use crate::app::ApplicationMessage;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::{Alignment, BorderSides, Borders, Color, Style},
    tui::{
        layout::Rect,
        text::{Line, Span, Text},
        widgets::{Block, Paragraph, Wrap},
        Frame,
    },
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State,
};

use super::DialogMessage;

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

impl Default for RemoveConfirmationDialog {
    fn default() -> Self {
        let mut properties = Props::default();
        let border = Borders::default()
            .color(Color::White)
            .sides(BorderSides::ALL);
        properties.set(Attribute::Borders, AttrValue::Borders(border));
        properties.set(Attribute::Background, AttrValue::Color(Color::Red));
        properties.set(Attribute::Foreground, AttrValue::Color(Color::White));
        properties.set(
            Attribute::Title,
            AttrValue::Title((String::from("Confirm delete"), Alignment::Left)),
        );
        properties.set(Attribute::HighlightedColor, AttrValue::Color(Color::Black));

        RemoveConfirmationDialog {
            file_list: Vec::new(),
            focused_button: Buttons::Cancel,
            properties,
        }
    }
}

pub struct RemoveConfirmationDialog {
    file_list: Vec<PathBuf>,
    focused_button: Buttons,
    properties: Props,
}

impl RemoveConfirmationDialog {
    pub fn new() -> Self {
        RemoveConfirmationDialog {
            file_list: Vec::new(),
            focused_button: Buttons::Cancel,
            properties: Props::default(),
        }
    }

    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.file_list = files;
        self
    }
}

impl MockComponent for RemoveConfirmationDialog {
    fn attr(&mut self, attr: Attribute, value: tuirealm::AttrValue) {
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
        let border = self
            .properties
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let background_color = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(Color::Red))
            .unwrap_color();
        let button_background = self
            .properties
            .get_or(Attribute::HighlightedColor, AttrValue::Color(Color::Black))
            .unwrap_color();
        let text_color = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(Color::White))
            .unwrap_color();
        let title = self
            .properties
            .get_or(
                Attribute::Title,
                AttrValue::Title(("Confirm delete".to_string(), Alignment::Left)),
            )
            .unwrap_title();
        let file_count = self
            .properties
            .get_or(Attribute::Value, AttrValue::Length(0))
            .unwrap_length();
        let text = self
            .properties
            .get_or(
                Attribute::HighlightedStr,
                AttrValue::String(format!(
                    "Are you sure you want to delete {} item(s)?",
                    file_count
                )),
            )
            .unwrap_string();

        let button_titles = {
            match self.focused_button {
                Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
                Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
            }
        };
        let button_styles = {
            match self.focused_button {
                Buttons::Ok => (
                    Style::default().bg(button_background).fg(text_color),
                    Style::default().fg(text_color),
                ),
                Buttons::Cancel => (
                    Style::default().fg(text_color),
                    Style::default().bg(button_background).fg(text_color),
                ),
            }
        };
        let spans = vec![
            Line::from(vec![Span::styled(text, Style::default().fg(text_color))]),
            Line::from(vec![Span::styled(
                "",
                Style::default()
                    .fg(text_color)
                    .add_modifier(tuirealm::tui::prelude::Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled(button_titles.0, button_styles.0),
                Span::styled(button_titles.1, button_styles.1),
            ]),
        ];
        let text = Text::from(spans);
        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .title(title.0)
                    .title_alignment(title.1)
                    .title_style(Style::default().fg(Color::Yellow))
                    .borders(border.sides)
                    .style(Style::default().bg(background_color).fg(text_color)),
            )
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }
}

impl Component<ApplicationMessage, NoUserEvent> for RemoveConfirmationDialog {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
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
            }) => {
                let msg = match self.focused_button {
                    Buttons::Ok => DialogMessage::RemoveSelectedFiles,
                    Buttons::Cancel => DialogMessage::CloseDialog,
                };

                Some(ApplicationMessage::Dialog(msg))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.focused_button.next();
                Some(ApplicationMessage::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.focused_button.next();
                Some(ApplicationMessage::None)
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                self.focused_button.next();
                Some(ApplicationMessage::None)
            }
            _ => None,
        }
    }
}
