use crate::app::ApplicationMessage;
use tui_realm_stdlib::Input;
use tuirealm::{
    command::{Cmd, CmdResult, Position},
    event::{Key, KeyEvent},
    props::{Alignment, BorderSides, Borders, Color, Style},
    tui::{
        layout::{Constraint, Layout, Rect},
        text::{Line, Span},
        widgets::{Block, Paragraph},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
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
                .borders(Borders::default().sides(BorderSides::NONE))
                .foreground(Color::White),
            _properties: Props::default(),
        }
    }
}

impl Component<ApplicationMessage, NoUserEvent> for MkDirDialog {
    fn on(&mut self, event: tuirealm::Event<NoUserEvent>) -> Option<ApplicationMessage> {
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
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(tuirealm::tui::layout::Direction::Vertical)
            .margin(1)
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

        let p = Paragraph::new("New directory name:")
            .style(Style::default().bg(Color::White).fg(Color::Black));
        frame.render_widget(p, dialog_layout[0]);

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

        self.component.view(frame, dialog_layout[1]);
        frame.render_widget(buttons, dialog_layout[3]);
    }
}

/*
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

#[derive(PartialEq)]
/// Represents the state of the dialog
pub enum MkDirDialogState {
    WaitingForInput,
    DisplayErrorMessage(String),
}

/// Represents a dialog used for creating a new directory.
pub struct MkDirDialog {
    button: Buttons,
    input: Input,
    hide: bool,
    parent_dir: PathBuf,
    state: MkDirDialogState,
}

impl MkDirDialog {
    pub fn new<P>(parent_dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        MkDirDialog {
            button: Buttons::Ok,
            input: Input::default(),
            hide: false,
            state: MkDirDialogState::WaitingForInput,
            parent_dir: PathBuf::from(parent_dir.as_ref()),
        }
    }

    pub fn create_dir(&mut self) -> io::Result<()> {
        let mut parent_dir = self.parent_dir.clone();
        parent_dir.push(self.input.value());
        std::fs::create_dir(parent_dir)
    }

    pub fn handle_key(&mut self, key: Key) {
        match self.state {
            MkDirDialogState::WaitingForInput => match key {
                Key::Char('\n') => match self.button {
                    Buttons::Ok => match self.create_dir() {
                        Ok(_) => self.hide = true,
                        Err(error) => {
                            self.state = MkDirDialogState::DisplayErrorMessage(error.to_string())
                        }
                    },
                    Buttons::Cancel => self.hide = true,
                },
                Key::Char(char) => {
                    // TODO: regex for allowed chars in linux file names
                    if char.is_alphanumeric() {
                        self.input.handle(InputRequest::InsertChar(char));
                    }
                }
                Key::Backspace => {
                    self.input.handle(InputRequest::DeletePrevChar);
                }
                Key::Delete => {
                    self.input.handle(InputRequest::DeleteNextChar);
                }
                Key::Right | Key::Left | Key::Up | Key::Down => self.button.next(),
                _ => {}
            },
            MkDirDialogState::DisplayErrorMessage(_) => match key {
                Key::Char('\n') => self.state = MkDirDialogState::WaitingForInput,
                Key::Esc => self.hide = true,
                _ => {}
            },
        }
    }

    /// Returns a representation based on the actual state of the dialog to render.
    pub fn widget(&self) -> Paragraph {
        match &self.state {
            MkDirDialogState::WaitingForInput => self.display_input(),
            MkDirDialogState::DisplayErrorMessage(msg) => self.display_error(msg),
        }
    }

    /// Signals that the dialog should be closed or not
    pub fn should_hide(&self) -> bool {
        self.hide
    }

    fn display_input(&self) -> Paragraph {
        let button_titles = match self.button {
            Buttons::Ok => ("[X] OK ", "[ ] Cancel"),
            Buttons::Cancel => ("[ ] OK ", "[X] Cancel"),
        };
        let spans = vec![
            Spans::from(vec![Span::styled(
                "New directory name:",
                Style::default().fg(Color::Black),
            )]),
            Spans::from(Span::styled(
                self.input.value(),
                Style::default().bg(Color::Cyan).fg(Color::Black),
            )),
            Spans::from(vec![
                Span::styled(button_titles.0, Style::default().fg(Color::Black)),
                Span::styled(button_titles.1, Style::default().fg(Color::Black)),
            ]),
        ];
        let text = Text::from(spans);
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Creating a new directory",
                        Style::default().fg(Color::Cyan),
                    ))
                    .style(Style::default().fg(Color::Black).bg(Color::Gray))
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Center),
            )
            .alignment(Alignment::Center)
    }

    fn display_error<'error_msg>(&self, error_message: &'error_msg str) -> Paragraph<'error_msg> {
        let spans = vec![
            Spans::from(vec![Span::styled(
                error_message,
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::styled(
                "[ OK ]",
                Style::default().fg(Color::White),
            )]),
        ];
        let text = Text::from(spans);
        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Error")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::LightRed).fg(Color::White)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::LightRed).fg(Color::Gray))
            .alignment(Alignment::Center)
    }
}*/
