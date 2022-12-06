use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{backend::TermionBackend, layout::{Rect}, Frame};

pub struct BottomMenu {}

impl BottomMenu {
    pub fn new() -> Self {
        BottomMenu {}
    }

    pub fn render(
        &self,
        main_layout: Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
        let menu_bottom_items = [
            " 1Help", " 2Menu", " 3View", " 4Edit", " 5Copy", " 6Move", " 7New", " 8Del", " 9Menu",
            "10Quit",
        ]
        .iter()
        .map(|item| {
            let (first, rest) = item.split_at(2);
            tui::text::Spans::from(vec![
                tui::text::Span::styled(
                    first,
                    tui::style::Style::default()
                        .fg(tui::style::Color::White)
                        .bg(tui::style::Color::Black),
                ),
                tui::text::Span::styled(
                    rest,
                    tui::style::Style::default()
                        .fg(tui::style::Color::Black)
                        .bg(tui::style::Color::Cyan),
                ),
            ])
        })
        .collect();
        let menu_bottom = tui::widgets::Tabs::new(menu_bottom_items)
            .style(tui::style::Style::default().bg(tui::style::Color::Cyan))
            .divider(tui::text::Span::raw(" "));
        frame.render_widget(menu_bottom, main_layout);
    }
}
