use super::{
    fixed_height_centered_rect, BottomMenu, CopyStrategy, Menu, MenuState, MkDirDialog,
    MoveStrategy, RmDirDialog, TableSortDirection, TableSortPredicate, TableView, TransferDialog,
};
use crate::app::{Application, InputMode};
use crate::core::config::Configuration;
use std::io::Stdout;
use std::path::PathBuf;
use std::rc::Rc;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Clear};
use tui::Frame;

#[derive(Copy, Clone)]
pub(crate) enum ActivePanel {
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
    Copy(TransferDialog<CopyStrategy>),
    Move(TransferDialog<MoveStrategy>),
    MkDir(MkDirDialog),
    RmDir(RmDirDialog),
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
    _config: Rc<Configuration>,
    dialog: Option<Dialog>,
    top_menu: MenuState,
    left_panel: TableView,
    right_panel: TableView,
    bottom_menu: BottomMenu,
    focused_widget: Widgets,
}

impl UserInterface {
    pub(crate) fn new(config: &Rc<Configuration>) -> Self {
        let left_table_config = config.left_table_config();
        let right_table_config = config.right_table_config();
        let mut left_panel = TableView::new(left_table_config);
        left_panel.activate();

        UserInterface {
            active_panel: ActivePanel::Left,
            _config: config.clone(),
            dialog: None,
            top_menu: MenuState::new_premade(),
            left_panel,
            right_panel: TableView::new(right_table_config),
            bottom_menu: BottomMenu::new(),
            focused_widget: Widgets::TwinPanel,
        }
    }

    pub(crate) fn left_table(&self) -> &TableView {
        &self.left_panel
    }

    pub(crate) fn right_table(&self) -> &TableView {
        &self.right_panel
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
                _ => (),
            },
            InputMode::Editing => {
                if let Some(dialog) = &mut self.dialog {
                    match dialog {
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
            }
        }
    }

    fn active_panel_mut(&mut self) -> &mut TableView {
        match &self.active_panel {
            ActivePanel::Left => &mut self.left_panel,
            ActivePanel::Right => &mut self.right_panel,
        }
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

    fn close_dialog(&mut self, app: &mut Application) {
        app.set_input_mode(InputMode::Normal);
        self.dialog = None;
        self.focused_widget = Widgets::TwinPanel;
    }
}
