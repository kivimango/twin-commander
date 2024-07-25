use crate::core::sort::{TableSortDirection, TableSortPredicate};
use crate::{
    app::ApplicationMessage,
    ui::{DialogMessage, PanelMessage, TopMenuMessage},
};
use tui_realm_stdlib::Table;
use tuirealm::{
    command::{Cmd, Direction},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, Borders, Color, TableBuilder},
    Component, Event, MockComponent, NoUserEvent,
};

/// Displays a directory's content with details in a table format.
#[derive(MockComponent)]
pub struct TableView {
    component: Table,
}

impl TableView {
    /// Creates a new TableView component.
    pub fn new() -> Self {
        let mut file_list = TableBuilder::default();

        TableView {
            component: Table::default()
                .background(Color::LightBlue)
                .borders(Borders::default())
                .column_spacing(0)
                .foreground(Color::White)
                .highlighted_color(Color::Red)
                .headers(&["Name", "Size", "Last Modified"])
                .rewind(false)
                .row_height(1)
                .scroll(true)
                .selected_line(0)
                .table(file_list.build())
                .title("", Alignment::Left)
                .widths(&[70, 10, 20]),
        }
    }
}

impl Component<ApplicationMessage, NoUserEvent> for TableView {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<ApplicationMessage> {
        let command = match event {
            // Bottom menu
            Event::Keyboard(KeyEvent {
                code: Key::Function(2),
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowHelpDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(7),
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowMkDirDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(8),
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Dialog(DialogMessage::ShowRmDialog)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(9),
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::TopMenu(TopMenuMessage::Focus)),
            Event::Keyboard(KeyEvent {
                code: Key::Function(10),
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Close),
            // Navigation
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Move(Direction::Up),
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => Cmd::Move(Direction::Down),
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                return Some(ApplicationMessage::Panel(PanelMessage::ChangeDirectory(
                    self.component.state(),
                )))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Panel(PanelMessage::GoBackUp)),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(ApplicationMessage::Panel(PanelMessage::SwitchPanel)),
            // Sorting
            Event::Keyboard(KeyEvent {
                code: Key::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::Name),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('l'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::LastModified),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortPredicate(TableSortPredicate::Size),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('u'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortDirection(TableSortDirection::Ascending),
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                return Some(ApplicationMessage::Panel(
                    PanelMessage::ChangeSortDirection(TableSortDirection::Descending),
                ))
            }
            _ => Cmd::None,
        };

        self.perform(command);
        Some(ApplicationMessage::None)
    }
}
