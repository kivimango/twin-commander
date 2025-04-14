use app::ApplicationModel;
use core::config;
use std::error::Error;
use tuirealm::terminal::TerminalBridge;

mod app;
mod core;
mod handlers;
mod ui;
mod worker_event;

fn main() -> Result<(), Box<dyn Error>> {
    // Initializing terminal with termion terminal backend and ratatui renderer
    let mut terminal = TerminalBridge::new()?;
    terminal.clear_screen()?;
    terminal.raw_mut().hide_cursor()?;

    // Initialize the app and run the event loop
    let config = config::get_config();
    let mut app = ApplicationModel::new(config);
    app.run(&mut terminal);

    config::save_config(app.get_config());

    // Restore terminal and close the application
    terminal.raw_mut().clear()?;
    terminal.raw_mut().show_cursor()?;
    Ok(())
}
