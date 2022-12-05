use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{
    backend::TermionBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Tabs,
    Frame,
};

const MENU_TITLES: [&str; 4] = ["Left", "File", "Command", "Right"];

enum MenuItems {
    Left,
    File,
    Command,
    Right,
}

impl From<&MenuItems> for usize {
    fn from(menu: &MenuItems) -> Self {
        match menu {
            MenuItems::Left => 0,
            MenuItems::File => 1,
            MenuItems::Command => 2,
            MenuItems::Right => 3,
        }
    }
}

impl Default for MenuItems {
    fn default() -> Self {
        MenuItems::Left
    }
}

impl MenuItems {
    fn next(&self) -> Self {
        match self {
            MenuItems::Left => MenuItems::File,
            MenuItems::File => MenuItems::Command,
            MenuItems::Command => MenuItems::Right,
            MenuItems::Right => MenuItems::Left,
        }
    }

    fn previous(&self) -> Self {
        match self {
            MenuItems::Left => MenuItems::Right,
            MenuItems::File => MenuItems::Left,
            MenuItems::Command => MenuItems::File,
            MenuItems::Right => MenuItems::Command,
        }
    }
}

/// Represents the menubar starting from the upper left corner.
pub struct Menu {
    selected: Option<MenuItems>,
}

impl Menu {
    pub fn new() -> Self {
        Menu { selected: None }
    }

    /// Renders the menu into the first row of the terminal
    pub fn render(
        &self,
        main_layout: Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
        let menu_items = MENU_TITLES
            .iter()
            .map(|item| {
                let (first, rest) = item.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::Black)),
                ])
            })
            .collect();
        let menu: Tabs;

        if let Some(selected) = &self.selected {
            menu = Tabs::new(menu_items)
                .select(selected.clone().into())
                .style(Style::default().bg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::White).bg(Color::Black))
                .divider(Span::raw(" "));
        } else {
            menu = Tabs::new(menu_items)
                .style(Style::default().bg(Color::Cyan))
                .divider(Span::raw(" "));
        }

        frame.render_widget(menu, main_layout);
    }

    pub fn select_previous(&mut self) {
        if let Some(selected) = &self.selected {
            self.selected = Some(selected.previous())
        }
    }

    pub fn select_next(&mut self) {
        if let Some(selected) = &self.selected {
            self.selected = Some(selected.next());
        } else {
            self.selected = Some(MenuItems::default());
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selected.is_some()
    }
}
