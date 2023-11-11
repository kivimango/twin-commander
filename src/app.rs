use crate::core::config::{self, try_load_from_file, try_save_to_file, Configuration};
use crate::event::{Event, Events};
use crate::ui::UserInterface;
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::Terminal;

/// Represents the various input modes that can be used in the application.
#[derive(Clone, Copy, Default)]
pub enum InputMode {
    /// This mode is used for normal operations and interactions within the application.
    /// This the default input mode when the application starts.
    #[default]
    Normal,

    /// The input mode used for editing.
    /// This mode is typically activated when the user is editing text or other content within the application.
    Editing,

    /// The input mode used for accessing the menu.
    /// This mode is active when the user is navigating or interacting with the application's menu system.
    Menu,
}

pub struct Application {
    input_mode: InputMode,
}

impl Application {
    /// Constructs a new instance of the `Application` struct with the default input mode.
    ///
    /// # Returns
    ///
    /// A new instance of the `Application` struct with the default input mode set.
    pub fn new() -> Self {
        Application {
            input_mode: InputMode::default(),
        }
    }

    // Runs the main event loop for the application, handling user input and updating the user interface accordingly.
    ///
    /// # Arguments
    ///
    /// * `terminal` - A mutable reference to the terminal instance used by the application.
    ///
    /// # Remarks
    ///
    /// This method continuously listens for user input and updates the user interface based on the current input mode.
    /// It also handles events such as quitting the application and saving the configuration before exiting.
    /// Performs initial checks for the configuration file and its path before starting the event loop.
    /// If the configuration file is not found, the application attempts to re-create it.
    /// Subsequently, the configuration data is loaded from the configuration file.
    pub(crate) fn run(&mut self, terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>) {
        let events = Events::new(None);
        let mut should_quit = false;
        let config = get_config();
        let mut ui = UserInterface::new(config);

        loop {
            if should_quit {
                break;
            }

            let _ignore = terminal.draw(|frame| {
                ui.draw(frame);
            });

            if let Ok(event) = events.recv() {
                match event {
                    Event::Input(key) => match &self.input_mode {
                        InputMode::Normal => match key {
                            Key::Esc | Key::F(10) => should_quit = true,
                            _ => {
                                ui.handle_key(key, self);
                            }
                        },
                        InputMode::Editing | InputMode::Menu => ui.handle_key(key, self),
                    },
                    Event::Tick => ui.tick(self),
                }
            }
        }
        // temporary solution to avoid Rc<RefCell<Configuration> everywhere in the ui
        ui.update_config();
        let config_to_save = ui.config();
        save_config(config_to_save);
    }

    /// Retrieves the current input mode of the application.
    ///
    /// # Returns
    ///
    /// The current input mode of the application.
    pub(crate) fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Sets the input mode of the application to the specified input mode.
    ///
    /// # Arguments
    ///
    /// * `input_mode` - The input mode to be set for the application.
    pub(crate) fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
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
