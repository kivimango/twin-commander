use super::TopMenuMessage;
use crate::app::ApplicationMessage;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Color, Style},
    tui::{
        layout::Rect,
        text::{Line, Span},
        widgets::Tabs,
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

pub struct BottomMenu {
    properties: Props,
    labels: [&'static str; 10],
}

impl BottomMenu {
    pub fn new() -> Self {
        BottomMenu {
            properties: Props::default(),
            labels: [
                " 1Help", " 2Menu", " 3View", " 4Edit", " 5Copy", " 6Move", " 7New", " 8Del",
                " 9Menu", "10Quit",
            ],
        }
    }

    /// Sets the background color of the buttons in the menu
    pub fn background(mut self, bacground: Color) -> Self {
        self.properties
            .set(Attribute::Background, AttrValue::Color(bacground));
        self
    }

    /// Sets the text color of the menu items in the menu
    pub fn label_foreground(mut self, foreground: Color) -> Self {
        self.properties
            .set(Attribute::Foreground, AttrValue::Color(foreground));
        self
    }

    /// Sets the function key highlight background color of the menu items in the menu
    pub fn function_key_background(mut self, background: Color) -> Self {
        self.properties
            .set(Attribute::HighlightedColor, AttrValue::Color(background));
        self
    }

    /// Sets the function key highlight text color of the menu items in the menu
    pub fn function_key_foreground(mut self, foreground: Color) -> Self {
        self.properties
            .set(Attribute::Color, AttrValue::Color(foreground));
        self
    }
}

impl MockComponent for BottomMenu {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value)
    }

    fn query(&self, query: Attribute) -> Option<AttrValue> {
        self.properties.get(query)
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let bacground = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(Color::Cyan))
            .unwrap_color();
        let foreground = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(Color::Black))
            .unwrap_color();
        let highlight_bg = self
            .properties
            .get_or(Attribute::HighlightedColor, AttrValue::Color(Color::Black))
            .unwrap_color();
        let higlight_fg = self
            .properties
            .get_or(Attribute::Color, AttrValue::Color(Color::White))
            .unwrap_color();

        let menu_bottom_items = self
            .labels
            .iter()
            .map(|item| {
                let (first, rest) = item.split_at(2);
                Line::from(vec![
                    Span::styled(first, Style::default().fg(higlight_fg).bg(highlight_bg)),
                    Span::styled(rest, Style::default().fg(foreground)),
                ])
            })
            .collect();

        let bottom_menu = Tabs::new(menu_bottom_items)
            .style(Style::default().bg(bacground))
            .divider(Span::raw(" "));

        frame.render_widget(bottom_menu, area);
    }
}

impl Component<ApplicationMessage, NoUserEvent> for BottomMenu {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(KeyEvent {
                code: Key::Function(9),
                modifiers: KeyModifiers::NONE,
            }) => Some(ApplicationMessage::TopMenu(TopMenuMessage::Focus)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(10),
                modifiers: KeyModifiers::NONE,
            }) => Some(ApplicationMessage::Close),
            _ => None,
        }
    }
}
