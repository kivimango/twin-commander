use crate::{app::ApplicationMessage, core::config::Configuration, ui::DialogMessage};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::Key,
    props::{Alignment, BorderType, Color, Style},
    tui::{
        layout::{Constraint, Direction, Layout},
        prelude::Rect,
        text::{Line, Span, Text},
        widgets::{Block, List, ListItem, ListState, Paragraph},
        Frame,
    },
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, State,
};

const CHECK_MARK: &str = "X";

enum Buttons {
    Apply,
    Cancel,
}

enum Components {
    OptionsList,
    Buttons,
}

impl Buttons {
    fn next(&mut self) -> Self {
        match self {
            Buttons::Apply => Buttons::Cancel,
            Buttons::Cancel => Buttons::Apply,
        }
    }
}

/// A dialog for changing the options shared by the left and right panel.
/// It is made up of a column and row containing the two buttons, Apply and Cancel respectively.
///
/// ## Key controls
/// Arrow keys:
/// * ↑ and ↓ : select options
/// * <- and -> : select left/right column
/// * Enter: change to the selected option
/// * Esc: closes the dialog without applying the changes to the configuration
pub struct PanelOpionsDialog {
    component: Components,
    focused_button: Buttons,
    list_state: ListState,
    options: [String; 1],
    request_config_change: bool,
    selected_option: usize,
    should_quit: bool,
    show_hidden_files: bool,
}

impl PanelOpionsDialog {
    pub fn new() -> Self {
        let mut options = [String::from("[ ] Show hidden files")];
        /*if config.show_hidden_files() {
            check_mark(&mut options[0])
        }*/

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        PanelOpionsDialog {
            component: Components::OptionsList,
            focused_button: Buttons::Cancel,
            list_state,
            options,
            request_config_change: false,
            selected_option: 0,
            should_quit: false,
            show_hidden_files: false,
        }
    }

    fn change_config(&mut self) {
        if self.selected_option == 0 {
            if self.show_hidden_files {
                self.show_hidden_files = false;
                uncheck_mark(&mut self.options[0]);
            } else {
                self.show_hidden_files = true;
                check_mark(&mut self.options[0]);
            }
        }
    }

    fn handle_keys(&mut self, key: Key) -> Option<ApplicationMessage> {
        match self.component {
            Components::Buttons => match key {
                Key::Up => {
                    self.component = Components::OptionsList;
                    self.list_state.select(Some(self.selected_option));
                    Some(ApplicationMessage::None)
                }
                Key::Left | Key::Right => {
                    self.focused_button = self.focused_button.next();
                    Some(ApplicationMessage::None)
                }
                Key::Char('\n') => {
                    match self.focused_button {
                        Buttons::Apply => self.apply(),
                        Buttons::Cancel => self.should_quit = true,
                    };
                    Some(ApplicationMessage::None)
                }
                _ => Some(ApplicationMessage::None),
            },
            Components::OptionsList => match key {
                Key::Up => {
                    self.select_previous_option();
                    Some(ApplicationMessage::None)
                }
                Key::Down => {
                    if self.selected_option == self.options.len() - 1 {
                        self.component = Components::Buttons;
                        self.list_state.select(None);
                    } else {
                        self.select_next_option();
                    }
                    Some(ApplicationMessage::None)
                }
                Key::Right => {
                    self.component = Components::Buttons;
                    self.list_state.select(None);
                    Some(ApplicationMessage::None)
                }
                Key::Char('\n') => {
                    self.change_config();
                    Some(ApplicationMessage::ConfigurationChanged(
                        crate::core::config::ConfigurationKey::ShowHiddenFiles(
                            self.show_hidden_files,
                        ),
                    ))
                }
                _ => Some(ApplicationMessage::None),
            },
        }
    }

    fn select_previous_option(&mut self) {
        if self.selected_option > 0 {
            self.selected_option -= 1;
            self.list_state.select(Some(self.selected_option));
        }
    }

    fn select_next_option(&mut self) {
        if self.selected_option < self.options.len() {
            self.selected_option += 1;
            self.list_state.select(Some(self.selected_option));
        }
    }

    fn apply(&mut self) {
        self.request_config_change = true;
        self.should_quit = true;
    }
}

impl Component<ApplicationMessage, NoUserEvent> for PanelOpionsDialog {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
        match event {
            Event::Keyboard(key_event) => match key_event.code {
                Key::Esc | Key::Function(10) => {
                    Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog))
                }
                _ => self.handle_keys(key_event.code),
            },
            _ => None,
        }
    }
}

impl MockComponent for PanelOpionsDialog {
    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }

    fn state(&self) -> State {
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let dialog_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
            .margin(1)
            .split(area);
        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Length(1),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(dialog_layout[0]);

        let items = ListItem::new(self.options[0].clone());
        let options_list = List::new(vec![items])
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::White));
        let mut options_list_state = self.list_state.clone();

        let buttons = buttons(&self.component, &self.focused_button);
        let block = Block::default()
            .borders(tuirealm::tui::widgets::Borders::ALL)
            .border_style(Style::default().fg(Color::Black))
            .border_type(BorderType::Plain)
            .title("Panel options")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(block, area);
        frame.render_stateful_widget(options_list, options_layout[0], &mut options_list_state);
        frame.render_widget(buttons, dialog_layout[1]);
    }
}

fn buttons(focused_component: &Components, focused_button: &Buttons) -> Paragraph<'static> {
    let focused_style = Style::default().bg(Color::Cyan).fg(Color::White);
    let button_style = Style::default().bg(Color::White);

    let button_styles = match focused_component {
        Components::OptionsList => (button_style, button_style),
        Components::Buttons => match focused_button {
            Buttons::Apply => (focused_style, button_style),
            Buttons::Cancel => (button_style, focused_style),
        },
    };
    let button_titles = {
        match focused_button {
            Buttons::Apply => ("[X] OK ", "[ ] Cancel"),
            Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
        }
    };
    let button_spans = Line::from(vec![
        Span::styled(button_titles.0, button_styles.0),
        Span::styled(button_titles.1, button_styles.1),
    ]);
    let button_text = Text::from(button_spans);
    Paragraph::new(button_text).alignment(Alignment::Center)
}

fn uncheck_mark(content: &mut String) {
    content.replace_range(1..2, " ");
}

fn check_mark(content: &mut String) {
    content.replace_range(1..2, CHECK_MARK);
}
