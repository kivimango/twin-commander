use std::io::Stdout;

use super::RenderWidget;
use termion::raw::RawTerminal;
use tui::{backend::TermionBackend, layout::Rect, Frame};

pub struct Panel<W: RenderWidget> {
    widget: W,
}

impl<W: RenderWidget> Panel<W> {
    /// Creates a Panel with the specified child widget.
    pub fn new(widget: W) -> Self {
        Panel { widget }
    }

    /// Renders the representation of the actual state into the terminal.
    /// The panel's visual representation is determined by the underlying child widget.
    pub fn render(&mut self, area: Rect, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>) {
        self.widget.render(area, frame);
    }
}