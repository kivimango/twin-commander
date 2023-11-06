use crate::core::config::{self, try_load_from_file, try_save_to_file, Configuration};
use crate::event::{Event, Events};
use crate::ui::UserInterface;
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::Terminal;

#[derive(Clone, Copy, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
    Menu,
}

pub struct Application {
    input_mode: InputMode,
}

impl Application {
    pub fn new() -> Self {
        Application {
            input_mode: InputMode::default(),
        }
    }

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
                            Key::Esc => should_quit = true,
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
        let config_to_save = ui.config();
        save_config(config_to_save);
    }

    pub(crate) fn input_mode(&self) -> InputMode {
        self.input_mode
    }

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
