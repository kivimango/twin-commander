use app::Application;
use std::error::Error;
use tuirealm::terminal::TerminalBridge;

mod app;
mod core;
mod event;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // Initializing terminal with termion terminal backend and ratatui renderer
    let mut terminal = TerminalBridge::new()?;
    terminal.clear_screen()?;
    terminal.raw_mut().hide_cursor()?;

    // Initialize the app and run the event loop
    let mut app = Application::new();
    app.run(&mut terminal);

    // Restore terminal and close the application
    terminal.raw_mut().clear()?;
    terminal.raw_mut().show_cursor()?;
    Ok(())
}
