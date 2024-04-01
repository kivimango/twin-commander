use super::DialogMessage;
use crate::app::ApplicationMessage;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::{Alignment, Color, Style},
    tui::{
        layout::{Constraint, Layout, Rect},
        style::Stylize,
        text::{Line, Span},
        widgets::{Block, Cell, Row},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

/// A simple dialog box to display key control/mapping information.
pub struct HelpDialog {
    properties: Props,
}

impl Component<ApplicationMessage, NoUserEvent> for HelpDialog {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog)),
            _ => None,
        }
    }
}

impl MockComponent for HelpDialog {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value)
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, query: Attribute) -> Option<AttrValue> {
        self.properties.get(query)
    }

    fn state(&self) -> State {
        State::None
    }

    /// Renders the help dialog on the specified frame and area.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable reference to the frame on which to render the help dialog.
    /// * `area` - The area where the help dialog should be rendered.
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let inner_area = Layout::default()
            .constraints([Constraint::Min(1), Constraint::Max(3)])
            .margin(1)
            .split(area);
        let button_area = Layout::default()
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .direction(tuirealm::tui::layout::Direction::Horizontal)
            .split(inner_area[1]);
        let style = Style::default().fg(Color::White);
        let bold_style = Style::default()
            .fg(Color::White)
            .add_modifier(tuirealm::tui::prelude::Modifier::BOLD);
        let key_style = Style::default().fg(Color::LightYellow);
        let block = tuirealm::tui::widgets::Block::default()
            .borders(tuirealm::tui::widgets::Borders::ALL)
            .bold()
            .title("Key Controls")
            .title_alignment(Alignment::Center);
        let header = Row::new(vec![
            Cell::from(Span::styled(" Menu", bold_style)),
            Cell::from(Span::styled("Panel", bold_style)),
            Cell::from(Span::styled("Application", bold_style)),
        ]);
        let rows = vec![
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("Select menuitem: ", style),
                    Span::styled(" 🡄 🡆", key_style),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("Change panel: ", style),
                    Span::styled("TAB", key_style),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("Exit: ", style),
                    Span::styled("F10", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("Select submenu: ", style),
                    Span::styled(" 🡅 🡇", key_style),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("Change directory: ", style),
                    Span::styled("Enter", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("Activate submenu: ", style),
                    Span::styled("Enter", key_style),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("Move cursor: ", style),
                    Span::styled(" 🡅 🡇", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled("Close menu: ", style),
                    Span::styled("Esc", key_style),
                ])),
                Cell::from(Line::from(vec![
                    Span::styled("Sort by name: ", style),
                    Span::styled("Ctrl+n", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Line::from(vec![
                    Span::styled("Sort by size: ", style),
                    Span::styled("Ctrl+s", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Line::from(vec![
                    Span::styled("Sort by last modified time: ", style),
                    Span::styled("Ctrl+l", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Line::from(vec![
                    Span::styled("Ascending order: ", style),
                    Span::styled("Ctrl+u", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Line::from(vec![
                    Span::styled("Descending order: ", style),
                    Span::styled("Ctrl+d", key_style),
                ])),
            ]),
        ];
        let widths = [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ];
        let table = tuirealm::tui::widgets::Table::new(rows, widths).header(header);
        let ok_button = tuirealm::tui::widgets::Paragraph::new("OK [ Enter ]")
            .alignment(Alignment::Center)
            .bold()
            .white()
            .block(Block::default().borders(tuirealm::tui::widgets::Borders::ALL));
        frame.render_widget(block, area);
        frame.render_widget(table, inner_area[0]);
        frame.render_widget(ok_button, button_area[1]);
    }
}

impl HelpDialog {
    /// Creates a new instance of `HelpDialog`.
    pub fn new() -> Self {
        HelpDialog {
            properties: Props::default(),
        }
    }
}