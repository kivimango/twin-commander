use crate::app::ApplicationMessage;
use std::borrow::Cow;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers},
    props::{BorderSides, BorderType, Color, Style},
    tui::{
        buffer::Buffer,
        layout::Rect,
        text::Span,
        widgets::{Block, Clear, StatefulWidget, Widget},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
    StateValue,
};

/// List of available messages that the top menu can produce to be handled by the model
#[derive(Debug, PartialEq)]
pub enum TopMenuMessage {
    /// Deactivates the top menu component
    Blur,

    /// Activates the top menu component
    Focus,
}

/// # TopMenu
/// Represents the menubar starting from the upper left corner.
///
/// ## Navigation
/// The top menu bar is designed to be navigated by keyboard.
///
/// * F9: Activates the menu: captures keyboard events
/// * Esc: deactivates the menu: key pressses no longer controls the menu
/// * ← →: when the top menu is activated, it can be navigated by the arrow keys.
///   Pressing the left arrow key will select the mostleft submenu from the current submenu,
///   pressing the right arrow key will select the next submenu from the current submenu.
/// * Enter: Pressing the Enter key on a submenu item, it will expand to show its menu items.
///   Pressing the Enter key on an expaned submenu's item will open a dialog of the selected menu item.
#[derive(MockComponent)]
pub struct TopMenu {
    component: MenuComponent,
}

impl TopMenu {
    /// Returns a new instance of a TopMenu with items and submenus prefilled.
    pub fn new() -> Self {
        TopMenu {
            component: MenuComponent::new(),
        }
    }

    /// Sets the top menu bar's background color.
    pub(crate) fn background(mut self, color: Color) -> Self {
        self.component
            .properties
            .set(Attribute::Background, AttrValue::Color(color));
        self
    }

    /// Sets the top menu bar's foreground (text) color.
    pub(crate) fn foreground(mut self, color: Color) -> Self {
        self.component
            .properties
            .set(Attribute::Foreground, AttrValue::Color(color));
        self
    }

    /// Sets the menu item's style in the top bar.
    pub(crate) fn item_style(mut self, item_style: Style) -> Self {
        self.component.properties.set(
            Attribute::Custom("item_style"),
            AttrValue::Style(item_style),
        );
        self
    }

    /// Sets the menu item's style when it is selected.
    /// This style is also used for the selected submenu item's style.
    pub(crate) fn selected_item_style(mut self, selected_item_style: Style) -> Self {
        self.component
            .properties
            .set(Attribute::FocusStyle, AttrValue::Style(selected_item_style));
        self
    }

    /*/// Sets the block around the submenu group when it is expanded.
    pub(crate) fn submenu_block(mut self, block: Block<'block>) -> Self {
        self.base.submenu_block = Some(block);
        self
    }*/
}

impl Component<ApplicationMessage, NoUserEvent> for TopMenu {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
        let command = match event {
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Function(9),
            }) => Cmd::Toggle,
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Function(10),
            }) => Cmd::Cancel,
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Esc,
            }) => Cmd::Cancel,
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Left,
            }) => Cmd::Move(Direction::Left),
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Right,
            }) => Cmd::Move(Direction::Right),
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Up,
            }) => Cmd::Move(Direction::Up),
            Event::Keyboard(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: Key::Down,
            }) => Cmd::Move(Direction::Down),
            _ => Cmd::None,
        };

        match self.perform(command) {
            CmdResult::Changed(State::One(StateValue::Bool(status))) => {
                if status {
                    Some(ApplicationMessage::TopMenu(TopMenuMessage::Focus))
                } else {
                    Some(ApplicationMessage::FocusBottomMenu)
                }
            }
            CmdResult::Changed(State::None) => Some(ApplicationMessage::Tick),
            _ => None,
        }
    }
}

/// A base menu component for storing style properties and implementing a custom renderer for the menu
struct MenuComponent {
    properties: Props,
    state: MenuState,
}

impl MenuComponent {
    fn new() -> Self {
        let mut properties = Props::default();
        properties.set(Attribute::Focus, AttrValue::Flag(false));
        MenuComponent {
            properties,
            state: MenuState::new_premade(),
        }
    }
}

impl MockComponent for MenuComponent {
    fn attr(&mut self, attribute: Attribute, value: AttrValue) {
        self.properties.set(attribute, value)
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Toggle => {
                let focused = self.query(Attribute::Focus).unwrap().unwrap_flag();
                if focused {
                    self.state.deactivate();
                    CmdResult::Changed(State::One(StateValue::Bool(false)))
                } else {
                    self.state.activate();
                    CmdResult::Changed(State::One(StateValue::Bool(true)))
                }
            }
            Cmd::Cancel => {
                self.state.deactivate();
                CmdResult::Changed(State::One(StateValue::Bool(false)))
            }
            Cmd::Move(direction) => {
                match direction {
                    Direction::Left => self.state.select_previous(),
                    Direction::Right => self.state.select_next(),
                    Direction::Up => self.state.up(),
                    Direction::Down => self.state.down(),
                }
                CmdResult::Changed(State::None)
            }
            Cmd::None => CmdResult::None,
            _ => CmdResult::None,
        }
    }

    fn query(&self, query: Attribute) -> Option<AttrValue> {
        self.properties.get(query)
    }

    fn state(&self) -> State {
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let bacground = self
            .properties
            .get_or(Attribute::Background, AttrValue::Color(Color::Cyan))
            .unwrap_color();
        let foreground = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(Color::White))
            .unwrap_color();
        let item_style = self
            .properties
            .get_or(
                Attribute::Custom("item_style"),
                AttrValue::Style(Style::default().bg(Color::Cyan).fg(Color::Gray)),
            )
            .unwrap_style();
        let selected_item_style = self
            .properties
            .get_or(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().bg(Color::Black).fg(Color::White)),
            )
            .unwrap_style();

        let widget = MenuRenderer {
            style: Style::default().bg(bacground).fg(foreground),
            item_style,
            selected_item_style,
            submenu_block: Some(
                Block::default()
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::White).bg(Color::Cyan))
                    .borders(BorderSides::all()),
            ),
        };
        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}

/// Keeps track of the state of the Menu preserved between two draw calls.
pub struct MenuState {
    items: Vec<SubMenu>,
    selected_item_idx: usize,
}

impl MenuState {
    fn new(items: Vec<SubMenu>) -> Self {
        MenuState {
            items,
            selected_item_idx: 0,
        }
    }

    /// Activates the menu: selects the first menu item,
    /// then select the first submenu item in this group.
    fn activate(&mut self) {
        self.selected_item_idx = 0;
        self.select_current();
        self.items[0].select_first();
    }

    /// Deselects the currently selected menu item.
    /// In the inactive state, menu group dropdowns are not drawned.
    fn deactivate(&mut self) {
        self.deselect_current()
    }

    /// Selects the previous menu item (from right to left).
    /// Calling this method wont has no effect when the currently selected menu item is the first.
    fn select_previous(&mut self) {
        if self.selected_item_idx == 0 {
            return;
        }

        self.deselect_current();
        self.selected_item_idx -= 1;
        self.select_current();
        self.items[self.selected_item_idx].select_first();
    }

    /// Selects the next menu item (from left to right).
    /// Calling this method has no effect when the currently selected menu item is the last.
    fn select_next(&mut self) {
        if self.selected_item_idx == self.items.len() - 1 {
            return;
        }

        self.deselect_current();
        self.selected_item_idx += 1;
        self.select_current();
        self.items[self.selected_item_idx].select_first();
    }

    /// Selects the submenu item one upper than the currently one (from bottom to top).
    /// Calling this method has no effect when the currently selected menu item is the first (the highest).
    fn up(&mut self) {
        if let Some(submenu) = self.items.get_mut(self.selected_item_idx) {
            if let Some(_item) = submenu.items.get_mut(submenu.highlighted_item_idx) {
                if submenu.highlighted_item_idx == 0 {
                    return;
                }

                submenu.deselect_current();
                submenu.highlighted_item_idx -= 1;
                submenu.select_current();
            }
        }
    }

    /// Selects the submenu item one lower than the currently one (from top to bottom).
    /// Calling this method has no effect when the currently selected menu item is the last (the lowest).
    fn down(&mut self) {
        if let Some(submenu) = self.items.get_mut(self.selected_item_idx) {
            if let Some(_item) = submenu.items.get_mut(submenu.highlighted_item_idx) {
                if submenu.highlighted_item_idx == submenu.items.len() - 1 {
                    return;
                }

                submenu.deselect_current();
                submenu.highlighted_item_idx += 1;
                submenu.select_current();
            }
        }
    }

    fn select_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_item_idx) {
            item.selected = true;
        }
    }

    fn deselect_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_item_idx) {
            item.selected = false;
        }
    }
    
    /// Creates a pre-made MenuState instance with submenus and its items filled.
    fn new_premade() -> Self {
        MenuState::new(vec![
            SubMenu::new(
                " Left ",
                vec![
                    MenuItem {
                        title: "Sort order".into(),
                        highlighted: false,
                    },
                    MenuItem {
                        title: "Filter".into(),
                        highlighted: false,
                    },
                ],
            ),
            SubMenu::new(
                " Options ",
                vec![MenuItem {
                    title: "Panel options".into(),
                    highlighted: false,
                }],
            ),
            SubMenu::new(
                " Right ",
                vec![
                    MenuItem {
                        title: "Sort order".into(),
                        highlighted: false,
                    },
                    MenuItem {
                        title: "Filter".into(),
                        highlighted: false,
                    },
                ],
            ),
        ])
    }
}

struct SubMenu {
    label: Cow<'static, str>,
    items: Vec<MenuItem>,
    highlighted_item_idx: usize,
    selected: bool,
    /// Width of the longest item title
    width: usize,
}

impl SubMenu {
    fn new(label: impl Into<Cow<'static, str>>, items: Vec<MenuItem>) -> Self {
        let longest_width = items
            .iter()
            .max_by(|x, y| x.title.len().cmp(&y.title.len()))
            .unwrap()
            .title
            .len();
        SubMenu {
            label: label.into(),
            items,
            highlighted_item_idx: 0,
            selected: false,
            width: longest_width,
        }
    }

    fn select_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.highlighted_item_idx) {
            item.highlighted = true;
        }
    }

    fn deselect_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.highlighted_item_idx) {
            item.highlighted = false;
        }
    }

    fn select_first(&mut self) {
        self.deselect_current();
        self.highlighted_item_idx = 0;
        self.select_current();
    }
}

struct MenuItem {
    title: Cow<'static, str>,
    highlighted: bool,
}

// An inbetween type for implementing a custom render method: in the view method,
// there is no access to the terminal buffer
struct MenuRenderer<'block> {
    style: Style,
    item_style: Style,
    selected_item_style: Style,
    submenu_block: Option<Block<'block>>,
}

impl<'block> StatefulWidget for MenuRenderer<'block> {
    type State = MenuState;

    fn render(mut self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        buffer.set_style(area, self.style);
        let mut x = area.left();
        let mut remaining_width = area.right().saturating_sub(x);

        for (_idx, submenu) in state.items.iter().enumerate() {
            let is_selected = submenu.selected;
            let has_children = !submenu.items.is_empty();

            let title_style = if submenu.selected {
                self.selected_item_style
            } else {
                self.style
            };

            if is_selected && has_children {
                let group_width = submenu.width as u16;
                self.render_dropdown(x, area.y + 1, &submenu.items, group_width, buffer);
            }

            let span = Span::styled(submenu.label.as_ref(), title_style);
            buffer.set_span(x, area.y, &span, remaining_width);
            x += span.width() as u16;

            remaining_width = remaining_width.saturating_sub(x);
        }
    }
}

impl<'block> MenuRenderer<'block> {
    fn render_dropdown(
        &mut self,
        x: u16,
        y: u16,
        group: &[MenuItem],
        group_width: u16,
        buffer: &mut Buffer,
    ) {
        let padding = 2;
        let area = Rect::new(x, y, group_width + padding, (group.len() as u16) + padding);
        let dropdown_area = match self.submenu_block.take() {
            Some(block) => {
                let inner_area = block.inner(area);
                Clear.render(area, buffer);
                block.render(area, buffer);
                buffer.set_style(inner_area, self.item_style);
                inner_area
            }
            None => area,
        };

        for (idx, item) in group.iter().enumerate() {
            let item_y = dropdown_area.top() + idx as u16;
            let item_style = if item.highlighted {
                self.selected_item_style
            } else {
                self.item_style
            };

            let span = Span::styled(item.title.as_ref(), item_style);
            buffer.set_span(dropdown_area.left(), item_y, &span, group_width);
        }
    }
}
