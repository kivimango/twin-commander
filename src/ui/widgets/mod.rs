use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{backend::TermionBackend, layout::Rect, Frame};

mod panel;
mod text_file_viewer;

pub use self::panel::*;
pub use self::text_file_viewer::*;



pub trait RenderWidget {
    /// Renders the representation of the actual state into the terminal.
    ///
    /// # Parameters
    /// * `area`: the available area to the widget for rendering its state
    /// * `frame`: the actual frame of the rendering loop
    fn render(&self, area: Rect, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>);
}
