use std::collections::HashSet;

use super::{panel_item::PanelItem, panel_state::PanelState};
use crate::core::list_dir::FilterOptions;
use crate::{
    app::ApplicationMessage,
    ui::{DialogMessage, PanelMessage, TableSortDirection, TableSortPredicate, TopMenuMessage},
    worker_event::WorkerEvent,
};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent},
    props::{Alignment, BorderSides, Borders, Color, PropValue},
    tui::{
        layout::{Constraint, Margin, Rect},
        style::Stylize,
        symbols::scrollbar,
        widgets::{
            Block, Cell, Row, ScrollDirection, Scrollbar, ScrollbarOrientation, ScrollbarState,
            Table, TableState,
        },
    },
    AttrValue, Attribute, Component, Event, MockComponent, Props, State, StateValue,
};
use tuirealm::{props::TextModifiers, tui::style::Style};

pub const SELECT_ITEM: &str = "SELECT_ITEM";
pub const CLEAR_SELECTION: &str = "CLEAR_ELECTION";

const DEFAULT_BACKGROUND_COLOR: Color = Color::LightBlue;
const DEFAULT_TEXT_COLOR: Color = Color::White;
const DEFAULT_FOCUS_STYLE: Style = Style {
    add_modifier: tuirealm::tui::prelude::Modifier::empty(),
    bg: Some(Color::Cyan),
    fg: Some(Color::Black),
    sub_modifier: tuirealm::tui::prelude::Modifier::empty(),
};

/// Represents a panel used to display the contents of a directory (files) in a table format.
/// It keeps track of the count of items in the table, along with its properties and state between draw calls.
pub struct TablePanel {
    /// Additional properties and settings for the table panel.
    properties: Props,

    /// The state keeps track of the cursor position and an offset from 0.
    table_state: TableState,

    // The other state of the panel preserved between two draw() calls
    state: PanelState,

    /// List of selected items
    selection: HashSet<usize>,

    /// The state of the scrollbar kept between draw calls
    srcoll_state: ScrollbarState,
}

impl Default for TablePanel {
    fn default() -> Self {
        let mut properties = Props::default();
        let border = Borders::default()
            .color(Color::White)
            .sides(BorderSides::ALL);
        properties.set(Attribute::Borders, AttrValue::Borders(border));
        properties.set(Attribute::Background, AttrValue::Color(Color::Blue));
        properties.set(Attribute::Foreground, AttrValue::Color(Color::White));
        properties.set(
            Attribute::Title,
            AttrValue::Title((String::new(), Alignment::Left)),
        );
        properties.set(
            Attribute::FocusStyle,
            AttrValue::Style(Style::default().bg(Color::Cyan).fg(Color::Black)),
        );

        let table_state = TableState::default().with_selected(Some(0));
        let state = PanelState::default();

        TablePanel {
            properties,
            table_state,
            state,
            selection: HashSet::new(),
            srcoll_state: ScrollbarState::default(),
        }
    }
}

impl Component<ApplicationMessage, WorkerEvent> for TablePanel {
    /// Key event handler: turns key presses into commands
    /// that the widget will process further.
    ///
    /// Key to Command Mappings:
    ///
    /// - `Function 2 Key`: Show help dialog
    /// - `Function 7 Key`: Show make directory dialog
    /// - `Function 8 Key`: Show remove dialog
    /// - `Function 9 Key`: Focus top menu
    /// - `Function 10 Key`: Close application
    /// - `Up Arrow Key`: Move up
    /// - `Down Arrow Key`: Move down
    /// - `Home Key`: Go to beginning of the list
    /// - `End Key`: Go to end of the list
    /// - `Enter Key`: Changes the current working directory into the selectem file (if it is a dir)
    /// - `Backspace Key`: Goes back to the parent directoy of the current working directory
    /// - `Tab Key`: Switches the current active panel to another
    /// - `Ctrl+n Key`: Changes sort predicate to name
    /// - `Ctrl+l Key`: Changes sort predicate to last modified
    /// - `Ctrl+s Key`: Changes sort predicate to size
    /// - `Ctrl+u Key`: Changes sort direction to ascending
    /// - `Ctrl+d Key`: Changes sort direction to descending
    /// - `Insert Key`: Marks the file at the cursor as selected, and advances the cursor by one row
    fn on(&mut self, event: Event<WorkerEvent>) -> Option<ApplicationMessage> {
        let cmd = match event {
            // Bottom menu
            Event::Keyboard(KeyEvent {
                code: Key::Function(2),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowHelpDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(5),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowCopyDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(6),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowMoveDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(7),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowMkDirDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(8),
                ..
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowRmDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(9),
                ..
            }) => return Some(ApplicationMessage::TopMenu(TopMenuMessage::Focus)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(10),
                ..
            }) => return Some(ApplicationMessage::Close),
            // Navigation
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => Cmd::Move(Direction::Up),
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => Cmd::Move(Direction::Down),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => Cmd::GoTo(Position::Begin),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => Cmd::GoTo(Position::End),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                return Some(ApplicationMessage::Panel(PanelMessage::ChangeDirectory(
                    self.state(),
                )))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => return Some(ApplicationMessage::Panel(PanelMessage::GoBackUp)),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(ApplicationMessage::Panel(PanelMessage::SwitchPanel))
            }
            // Sorting
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                ..
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::Name),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                ..
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::LastModified),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                ..
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::Size),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('u'),
                ..
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortDirection(TableSortDirection::Ascending),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortDirection(TableSortDirection::Descending),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Insert, ..
            }) => return Some(ApplicationMessage::Panel(PanelMessage::SelectItem)),
            Event::User(worker_event) => match worker_event {
                WorkerEvent::ListDirectoryResult(list_result) => {
                    match list_result {
                        Ok(files) => Cmd::None,
                        Err(_error) => Cmd::None, // show error message
                    }
                }
            },
            _ => Cmd::None,
        };

        self.perform(cmd);
        Some(ApplicationMessage::None)
    }
}

impl MockComponent for TablePanel {
    /// Sets  the table's properties based on the provided attribute and value.
    ///
    /// # Arguments
    ///
    /// * `attr` - The attribute to set.
    /// * `value` - The value corresponding to the attribute.
    ///
    /// # Accepted Properties and Values:
    ///
    /// - `Attribute::Content`: Sets the content of the widget, updating the row count if applicable.
    ///   - Value: `AttrValue::Table(Table)`, where `Table` represents the content data
    ///     in a two-dimensional array
    ///
    /// - `Attribute::Value`: Sets the value of the widget, typically used for selecting items.
    ///   - Value: `AttrValue::Payload(Payload)`, where `Payload` represents the data payload.
    ///     The payload should contain one item, which is the index of the selected item.
    ///
    /// - `Attribute::Background`: Sets the background color of the widget
    ///   - Value: `AttrValue::Color(Color)`, the color of the background
    ///
    /// - `Attribute::Foreground`: Sets the text color of the items in the rows of the widget
    ///   - Value: `AttrValue::Color(Color)`, the color of the item's text
    ///
    /// - `Attribute::Borders`: Sets the border around the widget
    ///   - Value: `AttrValue::Borders(Borders)`, the border width, color, sides and its modifiers around the widget
    ///
    /// - `Attribute::Title`: Sets the title of the table
    ///   - Value: `AttrValue::Title((String, Alingment))`, the current workind directory's path and its alignment
    ///
    /// - `Attribute::Text`: Sets the column headers of the table
    ///   - Value: `AttrValue::Text(Payload::Vec)`, the array of the column headers
    ///
    /// - `Attribute::FocusStyle`: Sets the selected item's style
    ///   - Value: `AttrValue::Style(Style)`, the style of the selected item.If the table is not focused, it will be set to the background and text color.
    ///
    /// - `Attribute::Custom(CLEAR_SELECTION)`: Marks all previously selected items as unselected.
    ///   - Value: `AttrValue::Flag(bool)`, currently the value is ignored.
    ///
    /// - `Attribute::Custom(SELECT_ITEM)`: Marks the item at the given index as selected.
    ///   - Value: `AttrValue::Payload(Payload::Usize)`, the index of the file to be selected.
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if matches!(attr, Attribute::Content) {
            // unwrapping the table attribute to query the row count in order to update it
            let table = value.unwrap_table();
            let row_count = table.len();
            //self.count = row_count;
            self.srcoll_state = ScrollbarState::new(row_count);
            self.properties.set(attr, AttrValue::Table(table));
        } else if matches!(attr, Attribute::Value) {
            let selected_idx = value.clone().unwrap_payload().unwrap_one().unwrap_usize();
            self.table_state.select(Some(selected_idx));
            self.properties.set(attr, value);
        } else if matches!(attr, Attribute::Custom(SELECT_ITEM)) {
            let selected_idx = value.unwrap_payload().unwrap_one().unwrap_usize();

            // selection / unselection
            if self.selection.contains(&selected_idx) {
                self.selection.remove(&selected_idx);
            } else {
                self.selection.insert(selected_idx);
            }
        } else if matches!(attr, Attribute::Custom(CLEAR_SELECTION)) {
            self.selection.clear();
        } else {
            self.properties.set(attr, value)
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        let count = self.state.count();

        match cmd {
            Cmd::Move(Direction::Down) => {
                if let Some(selected_idx) = self.table_state.selected() {
                    // TODO: could panic if count == 0
                    if count == 0 || selected_idx >= count - 1 {
                        return CmdResult::None;
                    }
                    if let Some(new_idx) = selected_idx.checked_add(1) {
                        self.table_state.select(Some(new_idx));
                        self.srcoll_state.scroll(ScrollDirection::Forward);
                        return CmdResult::Changed(self.state());
                    }
                    // TODO: could panic if selected_idx == Usize:MAX
                }
                CmdResult::None
            }
            Cmd::Move(Direction::Up) => {
                if let Some(selected_idx) = self.table_state.selected() {
                    if selected_idx == 0 || self.state.count() == 0 {
                        return CmdResult::None;
                    }
                    if let Some(new_idx) = selected_idx.checked_sub(1) {
                        self.table_state.select(Some(new_idx));
                        self.srcoll_state.scroll(ScrollDirection::Backward);
                        return CmdResult::Changed(self.state());
                    }
                }
                CmdResult::None
            }
            Cmd::GoTo(Position::Begin) => {
                if count != 0 {
                    self.table_state.select(Some(0));
                    self.srcoll_state.first();
                    return CmdResult::Changed(self.state());
                }
                CmdResult::None
            }
            Cmd::GoTo(Position::End) => {
                if count != 0 {
                    self.table_state.select(Some(count - 1));
                    self.srcoll_state.last();
                    return CmdResult::Changed(self.state());
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.properties.get(attr)
    }

    fn state(&self) -> State {
        if let Some(selected_idx) = self.table_state.selected() {
            State::One(StateValue::Usize(selected_idx))
        } else {
            State::None
        }
    }

    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let background = self
            .properties
            .get_or(
                Attribute::Background,
                AttrValue::Color(DEFAULT_BACKGROUND_COLOR),
            )
            .unwrap_color();
        let text_color = self
            .properties
            .get_or(Attribute::Foreground, AttrValue::Color(DEFAULT_TEXT_COLOR))
            .unwrap_color();
        let border = self
            .properties
            .get_or(Attribute::Borders, AttrValue::Borders(Borders::default()))
            .unwrap_borders();
        let title = self
            .properties
            .get_or(
                Attribute::Title,
                AttrValue::Title((String::new(), Alignment::Left)),
            )
            .unwrap_title();
        let headers = self
            .properties
            .get(Attribute::Text)
            .unwrap()
            .unwrap_payload()
            .unwrap_vec();

        // TODO: probably would be more efficient to store the file list as a Payload with referencing instead of cloning a table which is a 2D String vector
        //let files = self.properties.get_or(Attribute::Content, AttrValue::Payload(PropPayload::Vec(Vec::new()))).unwrap_payload().unwrap_vec();
        let files = self
            .properties
            .get(Attribute::Content)
            .unwrap()
            .unwrap_table();
        let focused = self
            .properties
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag();

        let (focused_style, title_style) = if focused {
            (
                self.properties
                    .get_or(Attribute::FocusStyle, AttrValue::Style(DEFAULT_FOCUS_STYLE))
                    .unwrap_style(),
                Style::default().bg(Color::White).fg(Color::Black),
            )
        } else {
            (
                Style::default().bg(background).fg(text_color),
                Style::default().black(),
            )
        };

        let header_titles: Vec<Cell> = headers
            .iter()
            .map(|value| match value {
                PropValue::Str(header_str) => Cell::new(header_str.as_str()).light_yellow().bold(),
                _ => Cell::new(""),
            })
            .collect();
        let headers = Row::new(header_titles);

        let rows: Vec<Row> = files
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let style = if self.selection.contains(&i) {
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(TextModifiers::BOLD)
                } else {
                    Style::default().fg(text_color)
                };
                Row::new([
                    Cell::from(f[0].content.clone()),
                    Cell::from(f[1].content.clone()),
                    Cell::from(f[2].content.clone()),
                ])
                .style(style)
            })
            .collect();

        let table = Table::default()
            .block(
                Block::default()
                    .borders(border.sides)
                    .title(title.0)
                    .title_alignment(title.1)
                    .title_style(title_style)
                    .style(Style::default().fg(border.color)),
            )
            .bg(background)
            .fg(text_color)
            .header(headers)
            .highlight_style(focused_style)
            .rows(rows)
            .widths([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(17),
            ]);

        frame.render_stateful_widget(table, area, &mut self.table_state);

        let table_height = area.height - 2;
        if self.state.count() > table_height.into() {
            let scroll_bar =
                Scrollbar::new(ScrollbarOrientation::VerticalRight).symbols(scrollbar::VERTICAL);
            let scroll_bar_area = area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            });
            frame.render_stateful_widget(scroll_bar, scroll_bar_area, &mut self.srcoll_state);
        }
    }
}
