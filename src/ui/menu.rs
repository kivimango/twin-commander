use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{
    backend::TermionBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};


/// Represents the menubar starting from the upper left corner.
pub struct Menu {}

impl Menu {
    pub fn new() -> Self {
        Menu {}
    }

    /// Renders the menu into the first row of the terminal
    pub fn render(
        &self,
        main_layout: Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
        let menu_spans = Spans::from(vec![
            Span::raw("Left"),
            Span::raw("    "),
            Span::raw("File"),
            Span::raw("    "),
            Span::raw("Command"),
            Span::raw("    "),
            Span::raw("Right"),
        ]);

        let menu_style = Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);
        let menu = Paragraph::new(menu_spans).style(menu_style);

        frame.render_widget(menu, main_layout);
    }
}
