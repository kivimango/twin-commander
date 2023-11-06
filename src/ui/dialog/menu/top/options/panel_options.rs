use crate::{
    app::Application,
    core::config::Configuration,
    ui::{user_interface::ActivePanel, BoxedDialog},
};
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

const CHECK_MARK: &'static str = "X";

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
    pub fn new(config: &Configuration) -> Self {
        let mut options = [String::from("[ ] Show hidden files")];
        if config.show_hidden_files() {
            check_mark(&mut options[0])
        }

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
            show_hidden_files: config.show_hidden_files(),
        }
    }

    fn change_config(&mut self) {
        match self.selected_option {
            0 => {
                if self.show_hidden_files {
                    self.show_hidden_files = false;
                    uncheck_mark(&mut self.options[0]);
                } else {
                    self.show_hidden_files = true;
                    check_mark(&mut self.options[0]);
                }
            }
            _ => {}
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

impl BoxedDialog for PanelOpionsDialog {
    fn change_configuration(&mut self, config: &mut Configuration, _activa_panel: ActivePanel) {
        config.set_show_hidden_files(self.show_hidden_files)
    }

    fn handle_keys(&mut self, key: Key, _app: &mut Application) {
        match self.component {
            Components::Buttons => match key {
                Key::Up => {
                    self.component = Components::OptionsList;
                    self.list_state.select(Some(self.selected_option));
                }
                Key::Left | Key::Right => self.focused_button = self.focused_button.next(),
                Key::Char('\n') => match self.focused_button {
                    Buttons::Apply => self.apply(),
                    Buttons::Cancel => self.should_quit = true,
                },
                _ => {}
            },
            Components::OptionsList => match key {
                Key::Up => self.select_previous_option(),
                Key::Down => {
                    if self.selected_option == self.options.len() - 1 {
                        self.component = Components::Buttons;
                        self.list_state.select(None);
                    } else {
                        self.select_next_option();
                    }
                }
                Key::Right => {
                    self.component = Components::Buttons;
                    self.list_state.select(None);
                }
                Key::Char('\n') => self.change_config(),
                _ => {}
            },
        }
    }

    fn render(
        &self,
        area: tui::layout::Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
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
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Black))
            .border_type(BorderType::Plain)
            .title("Panel options")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(block, area);
        frame.render_stateful_widget(options_list, options_layout[0], &mut options_list_state);
        frame.render_widget(buttons, dialog_layout[1]);
    }

    fn request_config_change(&self) -> bool {
        self.request_config_change
    }

    fn should_quit(&self) -> bool {
        self.should_quit
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
    let button_spans = Spans::from(vec![
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
