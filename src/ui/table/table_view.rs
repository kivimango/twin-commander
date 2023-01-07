use super::{
    centered_rect, sort, table_model::TableViewModel, TableSortDirection, TableSortPredicate,
};
use std::{
    io::Stdout,
    path::{Path, PathBuf},
};
use termion::raw::RawTerminal;
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

const CELL_HEADERS: [&str; 3] = ["Name", "Size", "Last modified"];

/// Displays a directory's content with details in a table format.
pub struct TableView {
    model: TableViewModel,
    is_active: bool,
    sort_direction: TableSortDirection,
    sort_predicate: TableSortPredicate,
}

impl TableView {
    /// Creates a new TableView instance with sane defaults.
    pub fn new() -> Self {
        TableView {
            model: TableViewModel::new(),
            is_active: false,
            sort_direction: TableSortDirection::default(),
            sort_predicate: TableSortPredicate::default(),
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn change_dir(&mut self) {
        if let Some(selected) = self.model.selected() {
            self.model.reset_selection();
            // the selected item is the parent of the cwd, go back up
            if selected == 0 {
                // the cwd is not the root dir
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
            self.sort();
            self.model.push_parent_front();
            self.select_first();
        }
    }

    pub fn get_selection(&self) -> Vec<PathBuf> {
        let mut selected = Vec::new();
        if let Some(selected_idx) = self.model.selected() {
            if let Some(file) = self.model.files().get(selected_idx) {
                let mut path = PathBuf::from(&self.model.pwd());
                path.push(PathBuf::from(&file.name));
                selected.push(path);
            }
        }
        selected
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn pwd(&self) -> &Path {
        self.model.pwd()
    }

    pub fn render_table(
        &mut self,
        main_layout: Rect,
        panel_idx: usize,
        frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>,
    ) {
        let twin_table_layout = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(tui::layout::Direction::Horizontal)
            .split(main_layout);

        let header_cells = CELL_HEADERS.iter().map(|header| Cell::from(*header));
        let table_header = Row::new(header_cells).height(1);
        let mut rows = Vec::new();
        let mut error = None;

        match self.model.list() {
            Ok(_) => {
                self.sort();
                self.model.push_parent_front();

                rows = self
                    .model
                    .files()
                    .iter()
                    .map(|row| {
                        Row::new(vec![
                            Cell::from(row.name.clone()),
                            Cell::from(row.size.clone()),
                            Cell::from(row.date.clone()),
                        ])
                    })
                    .collect::<Vec<Row>>();
            }
            Err(err) => {
                error = Some(err);
            }
        }

        let selected_style = match self.is_active {
            true => Style::default().fg(Color::Black).bg(Color::Red),
            false => Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::REVERSED),
        };
        let cwd = String::from(self.model.pwd().to_str().unwrap());

        let left_table = Table::new(rows)
            .block(Block::default().title(cwd).borders(Borders::ALL))
            .widths(&[
                Constraint::Percentage(70),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ])
            .header(table_header)
            .highlight_style(selected_style)
            .style(Style::default().bg(Color::LightBlue).fg(Color::White))
            .column_spacing(0);

        frame.render_stateful_widget(
            left_table,
            twin_table_layout[panel_idx],
            self.model.state_mut(),
        );

        if let Some(error) = error {
            let popup = Paragraph::new(error.to_string())
                .block(
                    Block::default()
                        .title("Error")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::LightRed).fg(Color::White)),
                )
                .wrap(Wrap { trim: false })
                .style(Style::default().bg(Color::LightRed).fg(Color::Gray))
                .alignment(Alignment::Center);
            let area = centered_rect(50, 25, twin_table_layout[panel_idx]);
            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
    }

    pub fn select_first(&mut self) {
        if self.model.files().is_empty() {
            return;
        };

        self.model.select(0);
    }

    pub fn select_last(&mut self) {
        if self.model.files().is_empty() {
            return;
        };

        self.model.select(self.model.files().len() - 1);
    }

    pub fn select_previous(&mut self) {
        self.model.select_previous();
    }

    pub fn select_next(&mut self) {
        self.model.select_next();
    }

    pub fn sort(&mut self) {
        sort(
            self.sort_direction,
            self.sort_predicate,
            self.model.files_mut(),
        );
    }

    pub fn set_sort_by(&mut self, direction: TableSortDirection) {
        self.sort_direction = direction;
    }
}
