use ::termion::raw::IntoRawMode;
use app::Application;
use std::{error::Error, io::stdout};
use tui::{backend::TermionBackend, Terminal};

mod app;
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

    let mut app = Application::new();
    app.run(&mut terminal);

    // Restore terminal and close the application
    terminal.clear()?;
    terminal.show_cursor()?;
    Ok(())
}
