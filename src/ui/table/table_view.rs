use super::{centered_rect, table_model::TableViewModel, TableSortDirection, TableSortPredicate};
use crate::{
    core::config::{Configuration, TableConfiguration},
    ui::RenderWidget,
};
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
    pub fn new(table_config: &TableConfiguration, config: &Configuration) -> Self {
        let mut model = TableViewModel::new(table_config, config);
        model.refresh();

        TableView {
            model,
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
        // remember current dir name before switching working dir
        let current_dir = PathBuf::from(self.model.pwd());
        let current_dir = current_dir.file_name();

        if self.model.cd().is_ok() && self.model.list().is_ok() {
            self.model.sort();
            self.model.push_parent_front();

            let file_count = self.model.files().len();
            if file_count <= 1 {
                self.select_first();
                return;
            }

            // select previous dir
            if let Some(current_dir_name) = current_dir {
                if let Some(previous_dir_index) = self
                    .model
                    .files()
                    .iter()
                    .filter(|f| f.is_dir)
                    .position(|f| f.name.as_str().eq(current_dir_name))
                {
                    if previous_dir_index <= file_count {
                        self.model.select(previous_dir_index);
                    }
                } else {
                    self.select_first();
                }
            }
        }
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
                let path = self.pwd().join(file.name.as_str());
                return Some(path);
            }
        }
        None
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

        if let Some(error) = self.model.last_error() {
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

        let file_list = self
            .model
            .files()
            .iter()
            .map(|file| {
                let cell_style = match file.is_dir {
                    true => Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                    false => Style::default().bg(Color::Blue).fg(Color::White),
                };
                let size_cell = match file.size {
                    Some(size) => Cell::from(format!("{}", SizeFormatter::new(size, DECIMAL))),
                    None => Cell::from("<DIR>"),
                };
                Row::new(vec![
                    Cell::style(Cell::from(file.name.clone()), cell_style),
                    Cell::style(size_cell, cell_style),
                    Cell::style(Cell::from(file.date.clone()), cell_style),
                ])
            })
            .collect::<Vec<Row>>();

        let selected_style = match self.is_active {
            true => Style::default().fg(Color::Black).bg(Color::Red),
            false => Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::REVERSED),
        };
        let cwd = String::from(self.model.pwd().to_str().unwrap());
        let name_column_width = table_layout[0].width - 3 - (8 + 16);
        let widths = [
            Constraint::Length(name_column_width),
            Constraint::Length(8),
            Constraint::Length(16),
        ];

        let table_view = Table::new(file_list)
            .block(Block::default().title(cwd).borders(Borders::ALL))
            .widths(&widths)
            .header(table_header)
            .highlight_style(selected_style)
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .column_spacing(0);

        frame.render_stateful_widget(table_view, table_layout[panel_idx], self.model.state_mut());
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
        self.sort();
    }

    pub fn sort_direction(&self) -> TableSortDirection {
        self.model.sort_direction()
    }

    pub fn sort_predicate(&self) -> TableSortPredicate {
        self.model.sort_predicate()
    }

    pub fn update_config(&mut self, new_config: TableConfiguration) {
        self.model
            .set_sort_predicate(TableSortPredicate::from(new_config.sort_predicate()));
        self.model
            .set_sort_direction(TableSortDirection::from(new_config.sort_direction()));
        self.model.refresh()
    }

    /// Sorts the table by the new `direction`.
    pub fn set_direction(&mut self, direction: TableSortDirection) {
        self.model.set_sort_direction(direction);
        self.model.sort();
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
