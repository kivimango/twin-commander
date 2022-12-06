use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::Terminal;
use crate::event::{Event, Events};
use crate::ui::{BottomMenu, Menu, TableView};

enum ActivePanel {
    Left,
    Right,
}

impl ActivePanel {
    fn switch(&mut self) {
        match self {
            ActivePanel::Left => *self = ActivePanel::Right,
            ActivePanel::Right => *self = ActivePanel::Left,
        }
    }
}

pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Application {}
    }

    pub(crate) fn run(&mut self, terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>) {
        let events = Events::new(None);
        let mut should_quit = false;

        let mut menu = Menu::new();
        let mut left_panel = TableView::new();
        let mut right_panel = TableView::new();
        left_panel.activate();
        let mut active_panel = ActivePanel::Left;
        let bottom_menu = BottomMenu::new();

        loop {
            if should_quit {
                break;
            }

            let _ignore = terminal.draw(|frame| {
                let frame_size = frame.size();

                let layout = Layout::default()
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Percentage(95),
                        Constraint::Min(1),
                    ])
                    .direction(tui::layout::Direction::Vertical)
                    .split(frame_size);

                menu.render(layout[0], frame);
                left_panel.render_table(layout[1], 0, frame);
                right_panel.render_table(layout[1], 1, frame);
                bottom_menu.render(layout[2], frame);
            });

            if let Ok(event) = events.recv() {
                match event {
                    Event::Input(key) => match key {
                        Key::Esc => should_quit = true,
                        Key::Char('\t') => {
                            if left_panel.is_active() {
                                left_panel.deactivate();
                                right_panel.activate();
                            } else {
                                left_panel.activate();
                                right_panel.deactivate();
                            }
                            active_panel.switch()
                        }
                        // Menu
                        Key::F(9) => menu.select_next(),
                        Key::Left => {
                            if menu.has_selection() {
                                menu.select_previous()
                            }
                        }
                        Key::Right => {
                            if menu.has_selection() {
                                menu.select_next()
                            }
                        }
                        Key::Home => match active_panel {
                            ActivePanel::Left => left_panel.select_first(),
                            ActivePanel::Right => right_panel.select_first(),
                        },
                        Key::End => match active_panel {
                            ActivePanel::Left => left_panel.select_last(),
                            ActivePanel::Right => right_panel.select_last(),
                        },
                        Key::Up => match active_panel {
                            ActivePanel::Left => left_panel.select_previous(),
                            ActivePanel::Right => right_panel.select_previous(),
                        },
                        Key::Down => match active_panel {
                            ActivePanel::Left => left_panel.select_next(),
                            ActivePanel::Right => right_panel.select_next(),
                        },
                        Key::Char('\n') => match active_panel {
                            ActivePanel::Left => left_panel.change_dir(),
                            ActivePanel::Right => right_panel.change_dir(),
                        },
                        _ => {}
                    },
                    Event::Tick => {}
                }
            }
        }
    }
}
