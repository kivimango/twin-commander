use super::{
    centered_rect, fixed_height_centered_rect, BottomMenu, BoxedDialog, CopyStrategy, HelpDialog,
    Menu, MenuState, MkDirDialog, MoveStrategy, PanelOpionsDialog, RmDirDialog, SortingDialog,
    TableSortDirection, TableSortPredicate, TableView, TransferDialog,
};
use crate::app::{Application, InputMode};
use crate::core::config::Configuration;
use std::io::Stdout;
use std::path::PathBuf;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Clear};
use tui::Frame;

#[derive(Copy, Clone)]
pub enum ActivePanel {
    Left,
    Right,
}

impl ActivePanel {
    fn switch(&mut self) {
        match self {
            ActivePanel::Left => *self = ActivePanel::Right,
            ActivePanel::Right => *self = ActivePanel::Left,
        }
    }
}

enum Dialog {
    Help(HelpDialog),
    Copy(TransferDialog<CopyStrategy>),
    Move(TransferDialog<MoveStrategy>),
    MkDir(MkDirDialog),
    RmDir(RmDirDialog),
    Menu(Box<dyn BoxedDialog>),
}

enum Widgets {
    TwinPanel,
    Dialog,
}

enum ShowDialogError {
    NoSelectedSource,
}

/// Represents the active state of the user interface.
/// It consists of three main parts:
/// * top menu: changes the left or right panel's directory view (list/tree TODO)
/// * the twin panel directory view: displays the contents of a `TableView::pwd()` path
/// * and the bottom menu: file operations
pub struct UserInterface {
    active_panel: ActivePanel,
    config: Configuration,
    dialog: Option<Dialog>,
    top_menu: MenuState,
    left_panel: TableView,
    right_panel: TableView,
    bottom_menu: BottomMenu,
    focused_widget: Widgets,
}

impl UserInterface {
    pub(crate) fn new(config: Configuration) -> Self {
        let (left_panel, right_panel) = {
            let left_table_config = config.left_table_config().clone();
            let right_table_config = config.right_table_config().clone();
            let mut left_panel = TableView::new(left_table_config.clone(), &config);
            left_panel.activate();
            (left_panel, TableView::new(right_table_config, &config))
        };

        UserInterface {
            active_panel: ActivePanel::Left,
            config,
            dialog: None,
            top_menu: MenuState::new_premade(),
            left_panel,
            right_panel,
            bottom_menu: BottomMenu::new(),
            focused_widget: Widgets::TwinPanel,
        }
    }

    pub(crate) fn config(&self) -> &Configuration {
        &self.config
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>) {
        let frame_size = frame.size();
        let layout = Layout::default()
            .constraints([
                Constraint::Min(1),
                Constraint::Percentage(95),
                Constraint::Min(1),
            ])
            .direction(tui::layout::Direction::Vertical)
            .split(frame_size);

        {
            self.left_panel.render_table(layout[1], 0, frame);
            self.right_panel.render_table(layout[1], 1, frame);
            self.bottom_menu.render(layout[2], frame);
        }

        {
            // Render top menu at last.
            // If expanded, menu will be drawn on top of content
            let top_menu = Menu::default()
                .style(Style::default().bg(Color::Cyan).fg(Color::White))
                .item_style(Style::default().bg(Color::Cyan).fg(Color::Gray))
                .selected_item_style(Style::default().bg(Color::Black).fg(Color::White))
                .submenu_block(
                    Block::default()
                        .border_type(tui::widgets::BorderType::Plain)
                        .border_style(Style::default().fg(Color::White).bg(Color::Cyan))
                        .borders(Borders::ALL),
                );
            frame.render_stateful_widget(top_menu, layout[0], &mut self.top_menu);
        }

        {
            // render dialog on top of content if it has...
            if let Some(dialog) = &mut self.dialog {
                match dialog {
                    Dialog::Help(help_dialog) => {
                        let area = fixed_height_centered_rect(75, 14, frame_size);
                        frame.render_widget(Clear, area);
                        help_dialog.render(frame, area);
                    }
                    Dialog::Copy(transfer_dialog) => {
                        let area = fixed_height_centered_rect(50, 8, frame_size);
                        frame.render_widget(Clear, area);
                        transfer_dialog.render(frame, area);
                    }
                    Dialog::Move(mv_dialog) => {
                        let area = fixed_height_centered_rect(50, 8, frame_size);
                        frame.render_widget(Clear, area);
                        mv_dialog.render(frame, area);
                    }
                    Dialog::MkDir(mkdir_dialog) => {
                        let area = fixed_height_centered_rect(33, 6, frame_size);
                        frame.render_widget(Clear, area);
                        frame.render_widget(mkdir_dialog.widget(), area);
                    }
                    Dialog::RmDir(rmdir_dialog) => {
                        let area = fixed_height_centered_rect(33, 7, frame_size);
                        frame.render_widget(Clear, area);
                        rmdir_dialog.render(frame, area);
                    }
                    Dialog::Menu(menu_dialog) => {
                        let area = centered_rect(33, 30, frame_size);
                        frame.render_widget(Clear, area);
                        menu_dialog.render(area, frame);
                    }
                }
            }
        }
    }

    pub(crate) fn handle_key(&mut self, key: Key, app: &mut Application) {
        let input_mode = app.input_mode();

        match input_mode {
            InputMode::Normal => match key {
                Key::Char('\t') => self.switch_focused_panel(),
                // Twin panel
                Key::Home => match &self.active_panel {
                    ActivePanel::Left => self.left_panel.select_first(),
                    ActivePanel::Right => self.right_panel.select_first(),
                },
                Key::End => match &self.active_panel {
                    ActivePanel::Left => self.left_panel.select_last(),
                    ActivePanel::Right => self.right_panel.select_last(),
                },
                Key::Up => match &self.active_panel {
                    ActivePanel::Left => self.left_panel.select_previous(),
                    ActivePanel::Right => self.right_panel.select_previous(),
                },
                Key::Down => match &self.active_panel {
                    ActivePanel::Left => self.left_panel.select_next(),
                    ActivePanel::Right => self.right_panel.select_next(),
                },
                Key::Char('\n') => match &self.active_panel {
                    ActivePanel::Left => self.left_panel.change_dir(),
                    ActivePanel::Right => self.right_panel.change_dir(),
                },
                // Tableview sorting by
                Key::Ctrl('n') => self.active_panel_mut().sort_by(TableSortPredicate::Name),
                Key::Ctrl('l') => self
                    .active_panel_mut()
                    .sort_by(TableSortPredicate::LastModified),
                Key::Ctrl('s') => self.active_panel_mut().sort_by(TableSortPredicate::Size),
                // Tableview sorting order
                Key::Ctrl('u') => self
                    .active_panel_mut()
                    .set_direction(TableSortDirection::Ascending),
                Key::Ctrl('d') => self
                    .active_panel_mut()
                    .set_direction(TableSortDirection::Descending),
                // Bottom menu
                Key::F(1) => {
                    app.set_input_mode(InputMode::Editing);
                    self.create_help_dialog();
                }
                // Copy file(s) dialog
                Key::F(5) => {
                    if let Ok(copy_dialog) = self.create_copy_dialog() {
                        self.dialog = Some(Dialog::Copy(copy_dialog));
                        self.focused_widget = Widgets::Dialog;
                        app.set_input_mode(InputMode::Editing);
                    }
                    // show error message about no selection
                }
                // Move file(s) dialog
                Key::F(6) => {
                    if let Ok(move_dialog) = self.create_move_dialog() {
                        self.dialog = Some(Dialog::Move(move_dialog));
                        self.focused_widget = Widgets::Dialog;
                        app.set_input_mode(InputMode::Editing);
                    }
                    // show error message about no selection
                }
                // Create directory dialog
                Key::F(7) => {
                    let parent_dir = match &self.active_panel {
                        ActivePanel::Left => self.left_panel.pwd(),
                        ActivePanel::Right => self.right_panel.pwd(),
                    };
                    self.dialog = Some(Dialog::MkDir(MkDirDialog::new(parent_dir)));
                    app.set_input_mode(InputMode::Editing);
                    self.focused_widget = Widgets::Dialog;
                }
                // Remove directory dialog
                Key::F(8) => {
                    if let Ok(rm_dialog) = self.create_rm_dialog() {
                        self.dialog = Some(Dialog::RmDir(rm_dialog));
                        app.set_input_mode(InputMode::Editing);
                        self.focused_widget = Widgets::Dialog;
                    }
                    // show error message about no selection
                }
                Key::F(9) => {
                    self.top_menu.activate();
                    app.set_input_mode(InputMode::Menu);
                }
                _ => (),
            },
            // Top menu
            InputMode::Menu => match key {
                Key::Left => self.top_menu.select_previous(),
                Key::Right => self.top_menu.select_next(),
                Key::Up => self.top_menu.up(),
                Key::Down => self.top_menu.down(),
                Key::F(9) | Key::Esc => {
                    self.top_menu.deactivate();
                    app.set_input_mode(InputMode::Normal)
                }
                Key::Char('\n') => {
                    app.set_input_mode(InputMode::Editing);
                    self.create_menu_dialog();
                }
                _ => (),
            },
            InputMode::Editing => {
                if let Some(dialog) = &mut self.dialog {
                    match dialog {
                        Dialog::Help(help_dialog) => {
                            help_dialog.handle_key(key);
                        }
                        Dialog::Copy(copy_dialog) => match key {
                            Key::Char('\n') => copy_dialog.handle_key(key),
                            Key::Esc => self.close_dialog(app),
                            _ => copy_dialog.handle_key(key),
                        },
                        Dialog::Move(mv_dialog) => match key {
                            Key::Char('\n') => mv_dialog.handle_key(key),
                            Key::Esc => self.close_dialog(app),
                            _ => mv_dialog.handle_key(key),
                        },
                        Dialog::MkDir(mkdir_dialog) => match key {
                            Key::Esc => self.close_dialog(app),
                            _ => mkdir_dialog.handle_key(key),
                        },
                        Dialog::RmDir(rmdir_dialog) => match key {
                            Key::Esc => self.close_dialog(app),
                            _ => rmdir_dialog.handle_keys(key),
                        },
                        Dialog::Menu(menu_dialog) => match key {
                            Key::Esc => self.close_dialog(app),
                            _ => menu_dialog.handle_keys(key, app),
                        },
                    }
                }
            }
        }
    }

    /// Switches the focus for the currently focused table panel to its counterpart
    /// (e.g. left=>right and left<=right).
    pub(crate) fn switch_focused_panel(&mut self) {
        if self.left_panel.is_active() {
            self.left_panel.deactivate();
            self.right_panel.activate();
        } else if self.right_panel.is_active() {
            self.right_panel.deactivate();
            self.left_panel.activate();
        }
        self.active_panel.switch();
    }

    /// Updates the ui's dialog if it has.
    pub(crate) fn tick(&mut self, app: &mut Application) {
        if let Some(dialog) = &mut self.dialog {
            match dialog {
                Dialog::Help(help_dialog) => {
                    if help_dialog.should_quit() {
                        self.close_dialog(app)
                    }
                }
                Dialog::Copy(copy_dialog) => {
                    copy_dialog.tick();
                    if copy_dialog.should_quit() {
                        self.close_dialog(app)
                    }
                }
                Dialog::Move(move_dialog) => {
                    move_dialog.tick();
                    if move_dialog.should_quit() {
                        self.close_dialog(app)
                    }
                }
                Dialog::MkDir(mk_dialog) => {
                    if mk_dialog.should_hide() {
                        self.close_dialog(app)
                    }
                }
                Dialog::RmDir(rm_dialog) => {
                    if rm_dialog.should_quit() {
                        self.close_dialog(app)
                    }
                }
                Dialog::Menu(dialog) => {
                    if dialog.should_quit() {
                        if dialog.request_config_change() {
                            let selected_menu_item = self.top_menu.selected_item();
                            match selected_menu_item {
                                // Left panel menu
                                0 => {
                                    dialog
                                        .change_configuration(&mut self.config, self.active_panel);
                                    self.left_panel
                                        .update_config(self.config.left_table_config());
                                }
                                // Panel options
                                1 => {
                                    dialog
                                        .change_configuration(&mut self.config, self.active_panel);
                                    self.left_panel.change_config(&self.config);
                                    self.right_panel.change_config(&self.config);
                                }
                                // Right panel menu
                                2 => {
                                    dialog
                                        .change_configuration(&mut self.config, self.active_panel);
                                    self.right_panel
                                        .update_config(self.config.right_table_config());
                                }
                                _ => {}
                            }
                        }
                        self.close_dialog(app)
                    }
                }
            }
        }
    }

    /// Collects the configuration values from widgets that may changed during runtime,
    /// and updates the current configuration with those changes.
    pub fn update_config(&mut self) {
        let left_path = PathBuf::from(self.left_panel.pwd());
        let left_sort_predicate = self.left_panel.sort_predicate();
        let left_sort_dir = self.left_panel.sort_direction();
        let right_path = PathBuf::from(self.right_panel.pwd());
        let right_sort_predicate = self.right_panel.sort_predicate();
        let right_sort_dir = self.right_panel.sort_direction();

        self.config.left_table_config_mut().set_path(left_path);
        self.config
            .left_table_config_mut()
            .set_predicate(left_sort_predicate.into());
        self.config
            .left_table_config_mut()
            .set_sort_direction(left_sort_dir.into());
        self.config.right_table_config_mut().set_path(right_path);
        self.config
            .right_table_config_mut()
            .set_predicate(right_sort_predicate.into());
        self.config
            .right_table_config_mut()
            .set_sort_direction(right_sort_dir.into());
    }

    fn active_panel_mut(&mut self) -> &mut TableView {
        match &self.active_panel {
            ActivePanel::Left => &mut self.left_panel,
            ActivePanel::Right => &mut self.right_panel,
        }
    }

    fn create_help_dialog(&mut self) {
        self.dialog = Some(Dialog::Help(HelpDialog::new()));
    }

    fn create_move_dialog(&self) -> Result<TransferDialog<MoveStrategy>, ShowDialogError> {
        match &self.active_panel {
            ActivePanel::Left => return inner(&self.left_panel, &self.right_panel),
            ActivePanel::Right => return inner(&self.right_panel, &self.left_panel),
        }

        fn inner(
            source: &TableView,
            target: &TableView,
        ) -> Result<TransferDialog<MoveStrategy>, ShowDialogError> {
            if let Some(selected_file) = source.get_selected_file() {
                let source = selected_file.as_path();
                let destination = target.pwd();
                Ok(TransferDialog::new(
                    PathBuf::from(source),
                    PathBuf::from(destination),
                    MoveStrategy,
                    String::from("Move file(s)"),
                ))
            } else {
                Err(ShowDialogError::NoSelectedSource)
            }
        }
    }

    fn create_copy_dialog(&self) -> Result<TransferDialog<CopyStrategy>, ShowDialogError> {
        match &self.active_panel {
            ActivePanel::Left => return inner(&self.left_panel, &self.right_panel),
            ActivePanel::Right => return inner(&self.right_panel, &self.left_panel),
        }

        fn inner(
            source: &TableView,
            target: &TableView,
        ) -> Result<TransferDialog<CopyStrategy>, ShowDialogError> {
            if let Some(selected_file) = source.get_selected_file() {
                let source = selected_file.as_path();
                let destination = target.pwd();
                Ok(TransferDialog::new(
                    PathBuf::from(source),
                    PathBuf::from(destination),
                    CopyStrategy,
                    String::from("Copy file(s)"),
                ))
            } else {
                Err(ShowDialogError::NoSelectedSource)
            }
        }
    }

    fn create_rm_dialog(&self) -> Result<RmDirDialog, ShowDialogError> {
        match &self.active_panel {
            ActivePanel::Left => return inner(&self.left_panel),
            ActivePanel::Right => return inner(&self.right_panel),
        }

        fn inner(source: &TableView) -> Result<RmDirDialog, ShowDialogError> {
            if let Some(selected_file) = source.get_selected_file() {
                let source = selected_file.as_path();
                Ok(RmDirDialog::new(vec![PathBuf::from(source)]))
            } else {
                Err(ShowDialogError::NoSelectedSource)
            }
        }
    }

    fn create_menu_dialog(&mut self) {
        let selectem_item_idx = self.top_menu.selected_item();
        match selectem_item_idx {
            0 => {
                let predicate = self.left_panel.sort_predicate();
                let direction = self.left_panel.sort_direction();
                self.dialog = Some(Dialog::Menu(Box::new(SortingDialog::new(
                    predicate, direction,
                ))));
            }
            1 => {
                let config = &self.config;
                self.dialog = Some(Dialog::Menu(Box::new(PanelOpionsDialog::new(config))));
            }
            2 => {
                let predicate = self.right_panel.sort_predicate();
                let direction = self.right_panel.sort_direction();
                self.dialog = Some(Dialog::Menu(Box::new(SortingDialog::new(
                    predicate, direction,
                ))));
            }
            _ => {}
        }
        self.top_menu.deactivate();
    }

    fn close_dialog(&mut self, app: &mut Application) {
        app.set_input_mode(InputMode::Normal);
        self.dialog = None;
        self.focused_widget = Widgets::TwinPanel;
    }
}
