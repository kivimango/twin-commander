use ::termion::raw::IntoRawMode;
use event::{Event, Events};
use std::{error::Error, io::stdout};
use termion::event::Key;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    Terminal,
};
use ui::{Menu, TableView};

mod core;
mod event;
mod ui;

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

fn main() -> Result<(), Box<dyn Error>> {
    // Initializing terminal with termion backend
    let stdout = stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut should_quit = false;

    let menu = Menu::new();
    let mut left_panel = TableView::new();
    let mut right_panel = TableView::new();
    left_panel.activate();
    let mut active_panel = ActivePanel::Left;

    loop {
        if should_quit {
            break;
        }

        terminal.draw(|frame| {
            let frame_size = frame.size();

            let layout = Layout::default()
                .constraints([Constraint::Min(1), Constraint::Percentage(99)])
                .direction(tui::layout::Direction::Vertical)
                .split(frame_size);

            menu.render(layout[0], frame);
            left_panel.render_table(layout[1], 0, frame);
            right_panel.render_table(layout[1], 1, frame);
        })?;

        if let Ok(event) = events.next() {
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

    // Restore terminal and close the application
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}
