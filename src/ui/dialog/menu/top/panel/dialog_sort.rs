use std::{io::Stdout, borrow::Cow};
use crate::{ui::{BoxedDialog, TableSortDirection, TableSortPredicate, user_interface::ActivePanel}, core::config::Configuration};
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph, List, ListItem, ListState}, backend::TermionBackend, Frame,
};

const CHECK_MARK: &'static str = "X";

enum Buttons {
    Apply,
    Cancel,
}

impl Buttons {
    fn next(&mut self) -> Self {
        match self {
            Buttons::Apply => Buttons::Cancel,
            Buttons::Cancel => Buttons::Apply,
        }
    }
}

enum Components {
    PredicateColumn,
    DirectionColumn,
    Buttons
}

struct PredicateList {
    predicate: TableSortPredicate,
    state: ListState,
    options: [String; 3],
    selected: usize,
}

impl PredicateList {
    fn new(predicate: TableSortPredicate) -> Self {
        PredicateList {
            predicate,
            state: ListState::default(),
            options: [
                "[ ] Name".into(),
                "[ ] Size".into(),
                "[ ] Last modified".into(),
            ],
            selected: predicate.to_usize()
        }
    }

    fn check_mark(&mut self) {
        let previous_predicate = self.predicate.to_usize();
        uncheck_mark(&mut self.options[previous_predicate]);
        self.predicate = TableSortPredicate::from(self.selected);
        check_mark(&mut self.options[self.selected]);
    }

    fn select(&mut self) {
        self.state.select(Some(self.selected))
    }

    fn unselect(&mut self) {
        self.state.select(None)
    }

    fn select_next(&mut self) {
        self.selected += 1;
        self.state.select(Some(self.selected));
    }

    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.state.select(Some(self.selected));
        }
    }
}

struct DirectionList {
    direction: TableSortDirection,
    state: ListState,
    options: [String; 2],
    selected: usize,
}

impl DirectionList {
    fn new(direction: TableSortDirection) -> Self {
        DirectionList {
            direction,
            state: ListState::default(),
            options: [
            "[ ] Ascending".into(),
            "[ ] Descending".into(),
            ],
            selected: direction.to_usize()
        }
    }

    fn check_mark(&mut self) {
        let previous_direction = self.direction.to_usize();
        uncheck_mark(&mut self.options[previous_direction]);
        self.direction = TableSortDirection::from(self.selected);
        check_mark(&mut self.options[self.selected]);
    }

    fn select(&mut self) {
        self.state.select(Some(self.selected))
    }

    fn unselect(&mut self) {
        self.state.select(None)
    }

    fn select_previous(&mut self) {
        if self.selected != 0 {
            self.selected -= 1;
            self.state.select(Some(self.selected));
        }
    }

    fn select_next(&mut self) {
        if self.selected != 1 {
            self.selected += 1;
            self.state.select(Some(self.selected));
        }
    }
}

/// A dialog for changing the currently focused TableView's sorting properties.
/// It is made up of two columns, on the left there is the PredicateList,
/// on the right is the DirectionList.
/// The bottom row contains the two buttons, Apply and Cancel respectively.
/// 
/// ## Key controls
/// Arrow keys:
/// * ↑ and ↓ : select options
/// * <- and -> : select left/right column
/// * Enter: change to the selected option
/// * Esc: closes the dialog without applying the changes to the configuration
pub struct SortingDialog {
    components: Components,
    change_config: bool,
    focused_button: Buttons,
    predicate_list: PredicateList,
    direction_list: DirectionList,
    should_quit: bool,
}

impl SortingDialog {
    /// Creates a new SortingDialog instance with the given configuration values.
    /// The left column is selected by default.
    pub fn new(predicate: TableSortPredicate, direction: TableSortDirection) -> Self {
        let mut predicate_list = PredicateList::new(predicate);
        let mut direction_list = DirectionList::new(direction);
        let components = Components::PredicateColumn;
        predicate_list.state.select(Some(predicate_list.selected));
        predicate_list.check_mark();
        direction_list.check_mark();
        
        SortingDialog {
            components,
            change_config: false,
            focused_button: Buttons::Cancel,
            predicate_list,
            direction_list,
            should_quit:false,
        }
    }

    fn apply(&mut self) {
        self.change_config = true;
        self.should_quit = true;
    }
}

impl BoxedDialog for SortingDialog {
    fn render(
        &self,
        area: Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>) {
        let dialog_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
            .margin(1)
            .split(area);
        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)].as_ref())
            .margin(1)
            .split(dialog_layout[0]);

        let button_titles = {
            match self.focused_button {
                Buttons::Apply => ("[X] OK ", "[ ] Cancel"),
                Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
            }
        };
        let button_styles = {
            let focused_style = Style::default().bg(Color::Cyan).fg(Color::White);
            let button_style = Style::default().bg(Color::White);
            match self.components {
                Components::PredicateColumn | Components::DirectionColumn => (button_style, button_style),
                Components::Buttons => {
                    match self.focused_button {
                        Buttons::Apply => (focused_style, button_style),
                        Buttons::Cancel => (button_style, focused_style),
                    }
                }   
            }
        };
        let button_spans = Spans::from(vec![
            Span::styled(button_titles.0, button_styles.0),
            Span::styled(button_titles.1, button_styles.1),
        ]);

        let mut left_list_state = self.predicate_list.state.clone();
        let left_items: Vec<ListItem<'_>> = self.predicate_list.options.iter().map(|item| ListItem::new(Cow::from(item))).collect();
        let left_list = List::new(left_items).highlight_style(Style::default().bg(Color::Cyan).fg(Color::White));

        let mut right_list_state = self.direction_list.state.clone();
        let right_items: Vec<ListItem> = self.direction_list.options.iter().map(|item| ListItem::new(Cow::from(item))).collect();
        let right_list = List::new(right_items).highlight_style(Style::default().bg(Color::Cyan).fg(Color::White));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Black))
            .border_type(BorderType::Plain)
            .title("Sorting mode")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::White).fg(Color::Black));

        let button_text = Text::from(button_spans);
        let buttons = Paragraph::new(button_text).alignment(Alignment::Center);

        frame.render_widget(block, area);
        frame.render_stateful_widget(left_list, options_layout[0], &mut left_list_state);
        frame.render_stateful_widget(right_list, options_layout[2], &mut right_list_state);
        frame.render_widget(buttons, dialog_layout[1]);
    }

    fn handle_keys(&mut self, key: Key, _app: &mut crate::app::Application) {
        match self.components {
            Components::PredicateColumn => {
                match key {
                    Key::Right => {
                        self.predicate_list.unselect();
                        self.direction_list.select();
                        self.components = Components::DirectionColumn;
                    }
                    Key::Up => self.predicate_list.select_previous(),
                    Key::Down => {
                        if self.predicate_list.selected == 2 {
                            self.predicate_list.unselect();
                            self.direction_list.select();
                            self.components = Components::DirectionColumn;
                        } else {
                            self.predicate_list.select_next();
                        }
                    }
                    Key::Char('\n') => self.predicate_list.check_mark(),
                    _ => {}
                }
            },
            Components::DirectionColumn => {
                match key {
                    Key::Left => {
                        self.direction_list.unselect();
                        self.predicate_list.select();
                        self.components = Components::PredicateColumn;
                    }
                    Key::Up => self.direction_list.select_previous(),
                    Key::Down => {
                        if self.direction_list.selected == 1 {
                            self.direction_list.unselect();
                            self.components = Components::Buttons;
                        } else {
                            self.direction_list.select_next();
                        }
                    }
                    Key::Char('\n') => self.direction_list.check_mark(),
                    _ => {}
                }
            },
            Components::Buttons => {
                match key {
                    Key::Left | Key::Right => self.focused_button = self.focused_button.next(),
                    Key::Up => {
                        self.predicate_list.select();
                        self.components = Components::PredicateColumn;
                    }
                    Key::Down => {
                        self.direction_list.select();
                        self.components = Components::DirectionColumn;
                    }
                    Key::Char('\n') => {
                        match self.focused_button {
                            Buttons::Apply =>  self.apply(),
                            Buttons::Cancel => {
                                self.change_config = false;
                                self.should_quit = true;
                            },
                        }
                    }
                    _ => {}
                }
            },
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn change_configuration(&mut self, config: &mut Configuration, active_panel: ActivePanel) {
        match active_panel {
            ActivePanel::Left => {
                config.left_table_config_mut().set_predicate(String::from(self.predicate_list.predicate));
                config.left_table_config_mut().set_sort_direction(String::from(self.direction_list.direction));
                eprintln!("{} {}", config.left_table_config().sort_predicate(), config.left_table_config().sort_direction());
            }
            ActivePanel::Right => {
                config.right_table_config_mut().set_predicate(String::from(self.predicate_list.predicate));
                config.right_table_config_mut().set_sort_direction(String::from(self.direction_list.direction));
                eprintln!("{} {}", config.right_table_config().sort_predicate(), config.right_table_config().sort_direction());
            }
        }
    }

    fn request_config_change(&self) -> bool {
        self.change_config
    }
}

fn uncheck_mark(content: &mut String) {
    content.replace_range(1..2, " ");
}

fn check_mark(content: &mut String) {
    content.replace_range(1..2, &CHECK_MARK);
}