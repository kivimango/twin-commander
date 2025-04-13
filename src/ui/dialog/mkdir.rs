use crate::{app::ApplicationMessage, worker_event::WorkerEvent};
use tui_realm_stdlib::Input;
use tuirealm::{
    command::{Cmd, CmdResult, Position},
    event::{Key, KeyEvent},
    props::{Alignment, BorderType, Borders, Color, InputType, Style},
    tui::{
        layout::{Constraint, Layout, Rect},
        text::{Line, Span},
        widgets::{Block, Paragraph},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State,
};

enum Buttons {
    Ok,
    Cancel,
}

impl Buttons {
    fn next(&mut self) {
        match *self {
            Buttons::Ok => *self = Buttons::Cancel,
            Buttons::Cancel => *self = Buttons::Ok,
        }
    }
}

/// Represents a dialog for creating a new directory, prompting the user to input the name of the new directory.
/// This dialog features an input field and two buttons, "Ok" and "Cancel", respectively.
/// When the user selects and confirms the directory name by pressing "Enter" or clicking "Ok",
/// the dialog sends a message to the application indicating the creation of the new directory,
/// with the input value as the name of the new directory.
/// Selecting and pressing "Enter" on "Cancel" closes the dialog without creating the directory.
pub struct MkDirDialog {
    /// The button component used to confirm directory creation.
    button: Buttons,

    /// The input component where the user can enter the name of the new directory.
    component: Input,

    /// Additional properties and settings for the dialog.
    _properties: Props,
}

impl MkDirDialog {
    pub fn new() -> Self {
        MkDirDialog {
            button: Buttons::Ok,
            component: Input::default()
                .background(Color::Cyan)
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::White),
                )
                .foreground(Color::White)
                .input_type(InputType::Text)
                .title("New directory name", Alignment::Left)
                .value("")
                .invalid_style(Style::default().fg(Color::Red)),
            _properties: Props::default(),
        }
    }
}

impl Component<ApplicationMessage, WorkerEvent> for MkDirDialog {
    fn on(&mut self, event: tuirealm::Event<WorkerEvent>) -> Option<ApplicationMessage> {
        let cmd = match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(ApplicationMessage::Dialog(
                    super::DialogMessage::CloseDialog,
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Function(10),
                ..
            }) => {
                return Some(ApplicationMessage::Dialog(
                    super::DialogMessage::CloseDialog,
                ))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.button {
                Buttons::Ok => Cmd::Submit,
                Buttons::Cancel => Cmd::Toggle,
            },
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => Cmd::Move(tuirealm::command::Direction::Left),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => Cmd::Move(tuirealm::command::Direction::Right),
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => Cmd::Cancel,
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => Cmd::Delete,
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => Cmd::GoTo(Position::Begin),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => Cmd::GoTo(Position::End),
            Event::Keyboard(KeyEvent {
                code: Key::Char(c), ..
            }) => Cmd::Type(c),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                self.button.next();
                Cmd::None
            }
            _ => Cmd::None,
        };

        match self.perform(cmd) {
            CmdResult::Submit(_) => Some(ApplicationMessage::Dialog(
                super::DialogMessage::CreateDirectory(self.state()),
            )),
            CmdResult::Custom("close_dialog") => Some(ApplicationMessage::Dialog(
                super::DialogMessage::CloseDialog,
            )),
            _ => None,
        }
    }
}

impl MockComponent for MkDirDialog {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value)
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Cancel => self.component.perform(cmd),
            Cmd::Delete => self.component.perform(cmd),
            Cmd::Move(_) => self.component.perform(cmd),
            Cmd::GoTo(_) => self.component.perform(cmd),
            Cmd::Type(_) => self.component.perform(cmd),
            Cmd::Submit => CmdResult::Submit(self.state()),
            Cmd::Toggle => CmdResult::Custom("close_dialog"),
            _ => CmdResult::None,
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.component.query(attr)
    }

    fn state(&self) -> State {
        self.component.state()
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let dialog_layout = Layout::default()
            .constraints([
                //Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(tuirealm::tui::layout::Direction::Vertical)
            .margin(2)
            .split(area);

        let block = Block::default()
            .title(Span::styled(
                "Create new directory",
                Style::default().fg(Color::LightBlue),
            ))
            .style(Style::default().fg(Color::Black).bg(Color::White))
            .borders(tuirealm::tui::widgets::Borders::ALL)
            .title_alignment(Alignment::Center);
        frame.render_widget(block, area);

        /*let p = Paragraph::new("New directory name:")
            .style(Style::default().bg(Color::White).fg(Color::Black));
        frame.render_widget(p, dialog_layout[0]);*/

        let button_titles = match self.button {
            Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
            Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
        };
        let button_styles = match self.button {
            Buttons::Ok => (
                Span::styled(
                    button_titles.0,
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
                Span::styled(button_titles.1, Style::default().fg(Color::Black)),
            ),
            Buttons::Cancel => (
                Span::styled(button_titles.0, Style::default().fg(Color::Black)),
                Span::styled(
                    button_titles.1,
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
            ),
        };
        let buttons = Line::from(vec![button_styles.0, button_styles.1]);
        let buttons = Paragraph::new(buttons).alignment(Alignment::Center);

        self.component.view(frame, dialog_layout[0]);
        frame.render_widget(buttons, dialog_layout[2]);
    }
}
