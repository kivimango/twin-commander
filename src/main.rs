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
    let mut table_view = TableView::new();

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
            table_view.render_table(layout[1], frame);
        })?;

        if let Ok(event) = events.next() {
            match event {
                Event::Input(key) => match key {
                    Key::Esc => should_quit = true,
                    Key::Home => table_view.select_first(),
                    Key::Up => table_view.select_previous(),
                    Key::Down => table_view.select_next(),
                    Key::Char('\n') => table_view.change_dir(),
                    _ => {}
                },
                Event::Tick => {}
            }
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}
