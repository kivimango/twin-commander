use super::table_model::TableViewModel;
use std::io::Stdout;
use termion::raw::RawTerminal;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

const CELL_HEADERS: [&str; 4] = ["Name", "Type", "Size", "Last modified"];

/// Displays a directory's content with details in a table format.
pub struct TableView {
    model: TableViewModel,
}

impl TableView {
    pub fn new() -> Self {
        TableView {
            model: TableViewModel::new(),
        }
    }

    pub fn render_table(
        &self,
        main_layout: Rect,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
        let twin_table_layout = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(tui::layout::Direction::Horizontal)
            .split(main_layout);

        let header_cells = CELL_HEADERS.iter().map(|header| Cell::from(*header));
        let table_header = Row::new(header_cells).height(1);
        let mut rowssw = Vec::new();

        if let Ok(rows) = self.model.list() {
            let rowss = rows
                .iter()
                .map(|row| {
                    Row::new(vec![
                        Cell::from(row.name.clone()),
                        Cell::from(row.ext.clone()),
                        Cell::from(row.size.clone()),
                        Cell::from(row.date.clone()),
                    ])
                })
                .collect::<Vec<Row>>();
            rowssw = rowss;
        }

        let left_table = Table::new(rowssw)
            .block(
                Block::default()
                    .title(self.model.pwd().to_str().unwrap())
                    .borders(Borders::ALL),
            )
            .widths(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .header(table_header)
            .style(Style::default().bg(Color::LightBlue).fg(Color::White))
            .column_spacing(0);
        let right_table = left_table.clone();

        frame.render_widget(left_table, twin_table_layout[0]);
        frame.render_widget(right_table, twin_table_layout[1]);
    }
}
