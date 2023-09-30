use crate::core::config::{self, try_load_from_file, try_save_to_file, Configuration};
use crate::event::{Event, Events};
use crate::ui::UserInterface;
use std::io::Stdout;
use std::rc::Rc;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::Terminal;

#[derive(Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
    Menu,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
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
        let config_to_save = get_actual_config(&ui);
        save_config(&config_to_save);
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

fn get_actual_config(ui: &UserInterface) -> Configuration {
    let mut config_to_save = Configuration::default();
    let left_path = ui.left_table().pwd().to_path_buf();
    let left_direction = ui.left_table().sort_direction();
    let left_predicate = ui.left_table().sort_predicate();
    let right_path = ui.right_table().pwd().to_path_buf();
    let right_direction = ui.right_table().sort_direction();
    let right_predicate = ui.right_table().sort_predicate();

    config_to_save.left_table_config_mut().set_path(left_path);
    config_to_save
        .left_table_config_mut()
        .set_sort_direction(left_direction.into());
    config_to_save
        .left_table_config_mut()
        .set_predicate(left_predicate.into());

    config_to_save.right_table_config_mut().set_path(right_path);
    config_to_save
        .right_table_config_mut()
        .set_sort_direction(right_direction.into());
    config_to_save
        .right_table_config_mut()
        .set_predicate(right_predicate.into());
    config_to_save
}
