use std::io::Stdout;
use termion::{event::Key, raw::RawTerminal};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

/// A simple dialog box to display key control/mapping information.
pub struct HelpDialog {
    should_quit: bool,
}

impl HelpDialog {
    /// Creates a new instance of `HelpDialog`.
    pub fn new() -> Self {
        HelpDialog { should_quit: false }
    }

    /// Handles the input key for the help dialog.
    /// If the key is the Enter key, F1 key, or Escape key, it sets the `should_quit` flag to true.
    pub fn handle_key(&mut self, key: Key) {
        if key == Key::Char('\n') || key == Key::F(1) || key == Key::Esc {
            self.should_quit = true
        }
    }

    /// Renders the help dialog on the specified frame and area.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable reference to the frame on which to render the help dialog.
    /// * `area` - The area where the help dialog should be rendered.
    pub fn render(&self, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>, area: Rect) {
        let inner_area = Layout::default()
            .constraints([Constraint::Min(1), Constraint::Max(1)])
            .margin(1)
            .split(area);
        let style = Style::default().fg(Color::White);
        let bold_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let key_style = Style::default().fg(Color::LightYellow);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Key Controls")
            .title_alignment(tui::layout::Alignment::Center);

        let header = Row::new(vec![
            Cell::from(Span::styled("Menu", bold_style)),
            Cell::from(Span::styled("Panel", bold_style)),
            Cell::from(Span::styled("Application", bold_style)),
        ]);
        let rows = vec![
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("Select menuitem: ", style),
                    Span::styled(" 🡄 🡆", key_style),
                ])),
                Cell::from(Spans::from(vec![
                    Span::styled("Change panel: ", style),
                    Span::styled("TAB", key_style),
                ])),
                Cell::from(Spans::from(vec![
                    Span::styled("Exit: ", style),
                    Span::styled("F10", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("Select submenu: ", style),
                    Span::styled(" 🡅 🡇", key_style),
                ])),
                Cell::from(Spans::from(vec![
                    Span::styled("Change directory: ", style),
                    Span::styled("Enter", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("Activate submenu: ", style),
                    Span::styled("Enter", key_style),
                ])),
                Cell::from(Spans::from(vec![
                    Span::styled("Move cursor: ", style),
                    Span::styled(" 🡅 🡇", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("Close menu: ", style),
                    Span::styled("Esc", key_style),
                ])),
                Cell::from(Spans::from(vec![
                    Span::styled("Sort by name: ", style),
                    Span::styled("Ctrl+n", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Spans::from(vec![
                    Span::styled("Sort by size: ", style),
                    Span::styled("Ctrl+s", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Spans::from(vec![
                    Span::styled("Sort by last modified time: ", style),
                    Span::styled("Ctrl+l", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Spans::from(vec![
                    Span::styled("Ascending order: ", style),
                    Span::styled("Ctrl+u", key_style),
                ])),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(Spans::from(vec![
                    Span::styled("Descending order: ", style),
                    Span::styled("Ctrl+d", key_style),
                ])),
            ]),
        ];

        let table = Table::new(rows).header(header).widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ]);

        let spacer = Paragraph::new(" ");
        let ok_button = Paragraph::new("OK [ Enter ]").alignment(tui::layout::Alignment::Center);

        let mut state = TableState::default();
        frame.render_widget(block, area);
        frame.render_stateful_widget(table, inner_area[0], &mut state);
        frame.render_widget(spacer, inner_area[0]);
        frame.render_widget(ok_button, inner_area[1]);
    }

    /// Returns a boolean indicating whether the help dialog should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_help_dialog() {
        let help_dialog = HelpDialog::new();
        assert_eq!(help_dialog.should_quit(), false);
    }

    #[test]
    fn test_handle_key_enter() {
        let mut help_dialog = HelpDialog::new();
        help_dialog.handle_key(Key::Char('\n'));
        assert_eq!(help_dialog.should_quit(), true);
    }

    #[test]
    fn test_handle_key_f1() {
        let mut help_dialog = HelpDialog::new();
        help_dialog.handle_key(Key::F(1));
        assert_eq!(help_dialog.should_quit(), true);
    }

    #[test]
    fn test_handle_key_esc() {
        let mut help_dialog = HelpDialog::new();
        help_dialog.handle_key(Key::Esc);
        assert_eq!(help_dialog.should_quit(), true);
    }

    #[test]
    fn test_handle_key_other() {
        let mut help_dialog = HelpDialog::new();
        help_dialog.handle_key(Key::Char('a'));
        assert_eq!(help_dialog.should_quit(), false);
    }
}
