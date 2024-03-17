use crate::core::config::{
    self, try_load_from_file, try_save_to_file, Configuration, TableConfiguration,
};
use crate::ui::{BottomMenu, TableView, TopMenu, TopMenuMessage};
use std::time::Duration;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Color, Style};
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::{
    AttrValue, Attribute, EventListenerCfg, NoUserEvent, PollStrategy, Sub, SubClause,
    SubEventClause, Update,
};

/// List of available user interface components in the application.
/// Variants are uniqe identifiers of those components used by tuirealm.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum UserInterfaces {
    Topmenu,
    LeftPanel,
    RightPanel,
    BottomMenu,
}

/// List of available messages the application can handle.
#[derive(Debug, PartialEq)]
pub enum ApplicationMessage {
    /// Requests closing the application
    Close,

    /// Brings up the top menu by stealing the focus from the currently focused component
    FocusBottomMenu,

    TopMenu(TopMenuMessage),

    /// Indicates that the component in current focus has handled its changes internally,
    /// and it wont send an application message, but the ui should be redrawn regardless
    Tick,
}

pub struct ApplicationModel {
    app: tuirealm::Application<UserInterfaces, ApplicationMessage, NoUserEvent>,
    should_quit: bool,
    redraw: bool,
}

impl ApplicationModel {
    pub fn new() -> Self {
        ApplicationModel {
            app: initialize(),
            should_quit: false,
            redraw: true,
        }
    }

    // Runs the main event loop for the application, handling user input and updating the user interface accordingly.
    ///
    /// # Arguments
    ///
    /// * `terminal` - A mutable reference to the terminal adapter instance used by the application.
    ///
    /// # Remarks
    ///
    /// This method continuously listens for user input and updates the user interface based on the current input mode.
    /// It also handles events such as quitting the application and saving the configuration before exiting.
    /// Performs initial checks for the configuration file and its path before starting the event loop.
    /// If the configuration file is not found, the application attempts to re-create it.
    /// Subsequently, the configuration data is loaded from the configuration file.
    pub fn run(&mut self, terminal: &mut TerminalBridge) {
        let config = get_config();
        mount_views(&mut self.app, config.left_table_config(), &config);

        while !self.should_quit {
            match self.app.tick(PollStrategy::Once) {
                Ok(messages) if !messages.is_empty() => {
                    self.redraw = true;
                    for message in messages {
                        let mut msg = Some(message);
                        while msg.is_some() {
                            msg = self.update(msg);
                        }
                    }
                }
                Ok(_) => {
                    self.redraw = true;
                }
                Err(error) => eprintln!("Error during tick: {error}"),
            }

            if self.redraw {
                self.view(terminal);
                self.redraw = false;
            }
        }

        save_config(&config);
    }

    fn view(&mut self, terminal: &mut TerminalBridge) {
        if let Err(error) = terminal.raw_mut().draw(|frame| {
            let frame_size = frame.size();
            let layout = Layout::default()
                .constraints([
                    Constraint::Min(1),
                    Constraint::Percentage(95),
                    Constraint::Min(1),
                ])
                .direction(Direction::Vertical)
                .split(frame_size);

            let table_layout = Layout::default()
                .constraints(&[Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(Direction::Horizontal)
                .split(layout[1]);

            self.app
                .view(&UserInterfaces::LeftPanel, frame, table_layout[0]);
            self.app
                .view(&UserInterfaces::RightPanel, frame, table_layout[1]);
            self.app.view(&UserInterfaces::BottomMenu, frame, layout[2]);

            // Draw menu at last to able to show expanded menus over content
            self.app.view(&UserInterfaces::Topmenu, frame, layout[0]);
        }) {
            eprint!("Error during drawing frame: {error}");
        }
    }
}

impl Update<ApplicationMessage> for ApplicationModel {
    fn update(&mut self, msg: Option<ApplicationMessage>) -> Option<ApplicationMessage> {
        if let Some(message) = msg {
            self.redraw = true;

            return match message {
                ApplicationMessage::Close => {
                    self.should_quit = true;
                    None
                }
                ApplicationMessage::FocusBottomMenu => {
                    self.app.active(&UserInterfaces::BottomMenu).unwrap();
                    Some(ApplicationMessage::Tick)
                }
                ApplicationMessage::TopMenu(top_menu_msg) => {
                    match top_menu_msg {
                        TopMenuMessage::Blur => {
                            self.app.active(&UserInterfaces::BottomMenu).unwrap();
                        }
                        TopMenuMessage::Focus => {
                            if let Some(focused_component) = self.app.focus() {
                                if !focused_component.eq(&UserInterfaces::Topmenu) {
                                    self.app.active(&UserInterfaces::Topmenu).unwrap();
                                }
                            }
                        }
                    }
                    Some(ApplicationMessage::Tick)
                }
                ApplicationMessage::Tick => None,
            };
        }
        None
    }
}

fn initialize() -> tuirealm::Application<UserInterfaces, ApplicationMessage, NoUserEvent> {
    tuirealm::Application::init(
        EventListenerCfg::default()
            .default_input_listener(Duration::from_millis(200))
            .poll_timeout(Duration::from_millis(200))
            .tick_interval(Duration::from_millis(60)),
    )
}

fn mount_views(
    app: &mut tuirealm::Application<UserInterfaces, ApplicationMessage, NoUserEvent>,
    table_config: &TableConfiguration,
    config: &Configuration,
) {
    let top_menu = TopMenu::new()
        .background(Color::Cyan)
        .foreground(Color::White)
        .item_style(Style::default().bg(Color::Cyan).fg(Color::Gray))
        .selected_item_style(Style::default().bg(Color::Black).fg(Color::White));
    let bottom_menu = BottomMenu::new()
        .background(Color::Cyan)
        .label_foreground(Color::Black)
        .function_key_background(Color::Black)
        .function_key_foreground(Color::White);
    let left_table = TableView::new(table_config, config);
    let right_table = TableView::new(table_config, config);

    app.mount(
        UserInterfaces::Topmenu,
        Box::new(top_menu),
        vec![
            Sub::new(
                SubEventClause::Any,
                SubClause::HasAttrValue(
                    UserInterfaces::Topmenu,
                    Attribute::Focus,
                    AttrValue::Flag(true),
                ),
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    code: Key::Function(9),
                }),
                SubClause::Always,
            ),
        ],
    )
    .expect("Failed to mount top menu component into the view!");
    app.mount(UserInterfaces::LeftPanel, Box::new(left_table), vec![])
        .expect("Failed to mount left tableview component into the view!");
    app.mount(UserInterfaces::RightPanel, Box::new(right_table), vec![])
        .expect("Failed to mount right tableview component into the view!");

    app.mount(UserInterfaces::BottomMenu, Box::new(bottom_menu), vec![])
        .expect("Failed to mount bottom menu component into the view!");
    app.active(&UserInterfaces::BottomMenu)
        .expect("Failed to activate bottom menu component!");
}

fn get_config() -> Configuration {
    let default_config = Configuration::default();

    if config::is_dir_exists() {
        if config::is_file_exists() {
            match try_load_from_file() {
                Ok(config) => config,
                Err(_) => default_config, // TODO: log error
            }
        } else {
            match config::try_save_to_file(&default_config) {
                Ok(_) => Configuration::default(),
                Err(_) => Configuration::default(),
            };
            default_config
        }
    } else if let Err(_error) = config::create_config_dir() {
        //TODO: log error
        default_config
    } else {
        match config::try_save_to_file(&default_config) {
            Ok(_) => default_config,
            Err(_) => default_config,
        }
    }
}

fn save_config(config: &Configuration) {
    if !config::is_dir_exists() {
        if config::create_config_dir().is_ok() {
            let _ = try_save_to_file(config);
        }
    } else {
        let _ = try_save_to_file(config);
    }
}
