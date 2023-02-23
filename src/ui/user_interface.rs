use super::{
    fixed_height_centered_rect, BottomMenu, CopyStrategy, Menu, MkDirDialog, MoveStrategy,
    RmDirDialog, TableSortDirection, TableSortPredicate, TableView, TransferDialog,
};
use crate::app::{Application, InputMode};
use std::io::Stdout;
use std::path::PathBuf;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::widgets::Clear;
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
    dialog: Option<Dialog>,
    top_menu: Menu,
    left_panel: TableView,
    right_panel: TableView,
    bottom_menu: BottomMenu,
    focused_widget: Widgets,
}

impl UserInterface {
    pub(crate) fn new() -> Self {
        let mut left_panel = TableView::new();
        left_panel.activate();

        UserInterface {
            active_panel: ActivePanel::Left,
            dialog: None,
            top_menu: Menu::new(),
            left_panel,
            right_panel: TableView::new(),
            bottom_menu: BottomMenu::new(),
            focused_widget: Widgets::TwinPanel,
        }
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
            self.top_menu.render(layout[0], frame);
            self.left_panel.render_table(layout[1], 0, frame);
            self.right_panel.render_table(layout[1], 1, frame);
            self.bottom_menu.render(layout[2], frame);
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
                // Top menu
                Key::Left => {
                    if self.top_menu.has_selection() {
                        self.top_menu.select_previous()
                    }
                }
                Key::Right => {
                    if self.top_menu.has_selection() {
                        self.top_menu.select_next()
                    }
                }
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
                Key::F(9) => self.top_menu.select_next(),
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
                            Key::Char('\n') => match mkdir_dialog.state() {
                                crate::ui::MkDirDialogState::WaitingForInput => {
                                    if mkdir_dialog.create_dir().is_ok() {
                                        self.close_dialog(app)
                                    }
                                    // show error message
                                }
                                crate::ui::MkDirDialogState::DisplayErrorMessage(_) => {
                                    self.close_dialog(app)
                                }
                                crate::ui::MkDirDialogState::DirCreated => self.close_dialog(app),
                            },
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
                Dialog::RmDir(rm_dialog) => {
                    if rm_dialog.should_quit() {
                        self.close_dialog(app)
                    }
                }
                _ => {}
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
