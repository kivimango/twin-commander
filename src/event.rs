use std::{
    io,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
use termion::{event::Key, input::TermRead};

pub const DEFAULT_TICK_RATE: u64 = 250;

/// Represents an event consumed by the application.
/// Event source is the termion backend.
pub enum Event<I> {
    /// A key input event signaling that the user pressed a key on the keyboard
    Input(I),
    /// A tick event.
    /// Event source sends ticks events perodically to the application.
    /// Tick events signals the application to refresh data and redraw the user interface with the new data.
    Tick,
}

/// Listent for key presses on the terminal,
/// and produces Events through a channel that can be consumed by the application.
pub struct Events {
    _tx: mpsc::Sender<Event<Key>>,
    rx: mpsc::Receiver<Event<Key>>,
}

impl Events {
    /// Creates a new Events instance that listens to key presses on a separate thread.
    /// It sends events through a channel back to the main thread for processing.
    pub fn new(tr: Option<u64>) -> Receiver<Event<Key>> {
        let tick_rate = match tr {
            Some(tick_rate) => Duration::from_millis(tick_rate),
            None => Duration::from_millis(DEFAULT_TICK_RATE),
        };

        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        thread::spawn(move || {
            let stdin = io::stdin();

            for key in stdin.keys().flatten() {
                if let Err(error) = tx.send(Event::Input(key)) {
                    // TODO: proper logging
                    eprintln!("Error during sending a key press event: {}", error);
                    return;
                }
            }
        });
        thread::spawn(move || loop {
            if let Err(error) = event_tx.send(Event::Tick) {
                eprintln!("Error during sending a tick event: {}", error);
                break;
            }
            thread::sleep(tick_rate);
        });
        rx
    }

    /*/// Attempts to read an event from the channel in a blocking way.
    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }*/
}
