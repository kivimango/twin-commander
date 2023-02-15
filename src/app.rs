use crate::event::{Event, Events};
use crate::ui::UserInterface;
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::Terminal;

#[derive(Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
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
        let mut ui = UserInterface::new();

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
                        InputMode::Editing => ui.handle_key(key, self),
                    },
                    Event::Tick => ui.tick(self),
                }
            }
        }
    }

    pub(crate) fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub(crate) fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
}
