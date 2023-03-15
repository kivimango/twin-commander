use super::{
    centered_rect, table_model::TableViewModel, TableSortDirection, TableSortPredicate, TableSorter,
};
use crate::core::config::TableConfiguration;
use humansize::{SizeFormatter, DECIMAL};
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

//const CELL_HEADERS: [&str; 3] = ["Name", "Size", "Last modified"];

const SORTED_BY_NAME_ASC: usize = 0;
const SORTED_BY_SIZE_ASC: usize = 1;
const SORTED_BY_LASTMODIFIED_ASC: usize = 2;
const SORTED_BY_NAME_DESC: usize = 3;
const SORTED_BY_SIZE_DESC: usize = 4;
const SORTED_BY_LASTMODIFIED_DESC: usize = 5;

const HEADER_LOOKUP_TABLE: [[&str; 3]; 6] = [
    ["Name▼", "Size", "Last modified"],
    ["Name", "Size▼", "Last modified"],
    ["Name", "Size", "Last modified▼"],
    ["Name▲", "Size", "Last modified"],
    ["Name", "Size▲", "Last modified"],
    ["Name", "Size", "Last modified▲"],
];

/// Displays a directory's content with details in a table format.
pub struct TableView {
    model: TableViewModel,
    is_active: bool,
}

impl TableView {
    /// Creates a new TableView instance with the provided configuration.
    pub fn new(table_config: &TableConfiguration) -> Self {
        TableView {
            model: TableViewModel::new(&table_config),
            is_active: false,
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;

        if !self.has_selection() {
            self.select_first()
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn change_dir(&mut self) {
        self.model.cd();
    }

    /*pub fn get_selection(&self) -> Option<usize> {
        // TODO: multi-select TableView
        let mut selected = Vec::new();
        if let Some(selected_idx) = self.model.selected() {
            if let Some(file) = self.model.files().get(selected_idx) {
                let mut path = PathBuf::from(&self.model.pwd());
                path.push(PathBuf::from(&file.name));
                selected.push(path);
            }
        }
        selected
    }*/

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        if let Some(idx) = self.model.selected() {
            if let Some(file) = self.model.files().get(idx) {
                return Some(self.pwd().join(file.name.clone()).to_path_buf());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn has_selection(&self) -> bool {
        self.model.selected().is_some()
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
        let table_layout = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(tui::layout::Direction::Horizontal)
            .split(main_layout);
        let header_cells = header_cells(self.model.sort_predicate(), self.model.sort_direction());
        let table_header = Row::new(header_cells).height(1);
        let mut file_list = Vec::new();
        let mut error = None;

        match self.model.list() {
            Ok(_) => {
                self.sort();
                self.model.push_parent_front();

                file_list = self
                    .model
                    .files()
                    .iter()
                    .map(|file| {
                        let cell_style = match file.is_dir {
                            true => Style::default()
                                .bg(Color::LightBlue)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                            false => Style::default().bg(Color::LightBlue).fg(Color::White),
                        };
                        let size_cell = match file.size {
                            Some(size) => {
                                Cell::from(format!("{}", SizeFormatter::new(size, DECIMAL)))
                            }
                            None => Cell::from("<DIR>"),
                        };
                        Row::new(vec![
                            Cell::style(Cell::from(file.name.clone()), cell_style),
                            Cell::style(size_cell, cell_style),
                            Cell::style(Cell::from(file.date.clone()), cell_style),
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

        let table_view = Table::new(file_list)
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

        frame.render_stateful_widget(table_view, table_layout[panel_idx], self.model.state_mut());

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
            let area = centered_rect(50, 25, table_layout[panel_idx]);
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
        self.model.sort();
    }

    /// Sorts the table by the new `predicate`.
    pub fn sort_by(&mut self, predicate: TableSortPredicate) {
        self.model.set_sort_predicate(predicate);
    }

    pub fn sort_direction(&self) -> TableSortDirection {
        self.model.sort_direction()
    }

    pub fn sort_predicate(&self) -> TableSortPredicate {
        self.model.sort_predicate()
    }

    /// Sorts the table by the new `direction`.
    pub fn set_direction(&mut self, direction: TableSortDirection) {
        self.model.set_sort_direction(direction);
    }
}

fn header_cells(
    sorted_by: TableSortPredicate,
    sort_order: TableSortDirection,
) -> impl Iterator<Item = Cell<'static>> {
    let header_lookup_index = match sorted_by {
        TableSortPredicate::Name => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_NAME_ASC,
            TableSortDirection::Descending => SORTED_BY_NAME_DESC,
        },
        TableSortPredicate::Size => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_SIZE_ASC,
            TableSortDirection::Descending => SORTED_BY_SIZE_DESC,
        },
        TableSortPredicate::LastModified => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_LASTMODIFIED_ASC,
            TableSortDirection::Descending => SORTED_BY_LASTMODIFIED_DESC,
        },
    };

    HEADER_LOOKUP_TABLE[header_lookup_index]
        .iter()
        .map(|header| Cell::from(*header))
}
