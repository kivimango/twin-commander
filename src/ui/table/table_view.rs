use super::table_model::TableViewModel;
use std::{io::Stdout, path::PathBuf};
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

    pub fn change_dir(&mut self) {
        if let Some(selected) = self.model.selected() {
            self.model.reset_selection();
            // go back up
            if selected == 0 {
                // if cwd is not the root dir
                if let Some(parent) = self.model.pwd().parent() {
                    self.model.set_cwd(parent.to_path_buf());
                    let _ = self.model.list();
                }
            }
            // change into the selected dir
            else {
                if let Some(file) = self.model.get_file(selected) {
                    let mut new_path = PathBuf::from(&self.model.pwd());
                    let dir_name = PathBuf::from(&file.name);
                    new_path.push(dir_name);
                    self.model.set_cwd(new_path);
                    let _ = self.model.list();
                }
            }
        }
    }

    pub fn render_table(
        &mut self,
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

        if let Ok(_) = self.model.list() {
            let rowss = self
                .model
                .files()
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

        let selected_style = Style::default().fg(Color::Black).bg(Color::Red);
        let cwd = String::from(self.model.pwd().to_str().unwrap());

        let left_table = Table::new(rowssw)
            .block(Block::default().title(cwd).borders(Borders::ALL))
            .widths(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .header(table_header)
            .highlight_style(selected_style)
            .style(Style::default().bg(Color::LightBlue).fg(Color::White))
            .column_spacing(0);
        let right_table = left_table.clone();

        frame.render_stateful_widget(left_table, twin_table_layout[0], self.model.state_mut());
        frame.render_widget(right_table, twin_table_layout[1]);
    }

    pub fn select_previous(&mut self) {
        self.model.select_previous();
    }

    pub fn select_next(&mut self) {
        self.model.select_next();
    }
}
