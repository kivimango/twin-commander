use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{backend::TermionBackend, layout::Rect, Frame};

mod panel;
mod text_file_viewer;

pub use self::panel::*;
pub use self::text_file_viewer::*;

/// A list of available widgets to use in a `Panel`.
pub enum Widgets {
    /// Displays the content of the working directory in a table-like format.
    Table,
    /// Displays the contents of a text file.
    TextFileViewer,
}

pub trait RenderWidget {
    /// Renders the representation of the actual state into the terminal.
    ///
    /// # Parameters
    /// * `area`: the available area to the widget for rendering its state
    /// * `frame`: the actual frame of the rendering loop
    fn render(&self, area: Rect, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>);
}
