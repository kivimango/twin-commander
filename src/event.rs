use std::{sync::mpsc, thread, io};
use termion::{event::{Key}, input::TermRead};

/// Represents an event consumed by the application.
/// Event source is the termion backend.
pub enum Event<I> {
    /// A key input event signaling that the user pressed a key on the keyboard
    Input(I),
    /// A tick event.
    /// Event source sends ticks events perodically to the application.
    /// Tick events signals the application to refresh data and redraw the user interface with the new data.
    Tick
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
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        thread::spawn(move || {
            //maybe use termion::async_stdin() ?
            let stdin = io::stdin();
            let mut keys = stdin.keys();

            loop {
                let key_event = keys.next();

                if let Some(key_event) = key_event {
                    if let Ok(key) = key_event {
                        event_tx.send(Event::Input(key)).unwrap();
                    }
                }

                event_tx.send(Event::Tick).unwrap();
            }
        });
        Events {
            rx,
            _tx: tx
        }
    }

    /// Attempts to read an event from the channel in a blocking way.
    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}