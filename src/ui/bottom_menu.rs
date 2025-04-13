use crate::{app::ApplicationMessage, worker_event::WorkerEvent};
use tuirealm::{
    command::{Cmd, CmdResult},
    props::{Color, Style},
    tui::{
        layout::Rect,
        text::{Line, Span},
        widgets::Tabs,
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State,
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

        let menu_bottom_items: Vec<Line> = self
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
            .highlight_style(Style::default())
            .style(Style::default().bg(bacground))
            .divider(Span::raw(" "));

        frame.render_widget(bottom_menu, area);
    }
}

impl Component<ApplicationMessage, WorkerEvent> for BottomMenu {
    fn on(&mut self, _event: Event<WorkerEvent>) -> Option<ApplicationMessage> {
        None
    }
}
