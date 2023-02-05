use crate::event::{Event, Events};
use crate::ui::{
    centered_rect, fixed_height_centered_rect, BottomMenu, CopyDialog, Menu, MkDirDialog,
    RmDirDialog, TableView,
};
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::widgets::Clear;
use tui::Terminal;

enum Dialog {
    CopyDialog(CopyDialog),
    MkDirDialog(MkDirDialog),
    RmDirDialog(RmDirDialog),
}

enum Widgets {
    TwinPanel,
    Dialog,
}

enum InputMode {
    Normal,
    Editing,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

enum ActivePanel {
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

pub struct Application {
    dialog: Option<Dialog>,
    input_mode: InputMode,
    focused_widget: Widgets,
}

impl Application {
    pub fn new() -> Self {
        Application {
            dialog: None,
            input_mode: InputMode::default(),
            focused_widget: Widgets::TwinPanel,
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut Terminal<TermionBackend<RawTerminal<Stdout>>>) {
        let events = Events::new(None);
        let mut should_quit = false;

        let mut menu = Menu::new();
        let mut left_panel = TableView::new();
        let mut right_panel = TableView::new();
        left_panel.activate();
        let mut active_panel = ActivePanel::Left;
        let bottom_menu = BottomMenu::new();

        loop {
            if should_quit {
                break;
            }

            let _ignore = terminal.draw(|frame| {
                let frame_size = frame.size();

                let layout = Layout::default()
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Percentage(95),
                        Constraint::Min(1),
                    ])
                    .direction(tui::layout::Direction::Vertical)
                    .split(frame_size);

                menu.render(layout[0], frame);
                left_panel.render_table(layout[1], 0, frame);
                right_panel.render_table(layout[1], 1, frame);
                bottom_menu.render(layout[2], frame);

                if let Some(dialog) = &self.dialog {
                    match dialog {
                        Dialog::CopyDialog(cp_dialog) => {
                            let area = fixed_height_centered_rect(50, 8, frame_size);
                            frame.render_widget(Clear, area);
                            cp_dialog.render(frame, area);
                        }
                        Dialog::MkDirDialog(mkdir_dialog) => {
                            let area = centered_rect(33, 20, frame_size);
                            frame.render_widget(Clear, area);
                            frame.render_widget(mkdir_dialog.widget(), area);
                        }
                        Dialog::RmDirDialog(rmdir_dialog) => {
                            if rmdir_dialog.should_quit() {
                                self.input_mode = InputMode::Normal;
                                self.dialog = None;
                                self.focused_widget = Widgets::TwinPanel;
                                return;
                            }
                            let area = fixed_height_centered_rect(33, 6, frame_size);
                            frame.render_widget(Clear, area);
                            rmdir_dialog.render(frame, area);
                        }
                    }
                }
            });

            if let Ok(event) = events.recv() {
                match event {
                    Event::Input(key) => match &self.input_mode {
                        InputMode::Normal => {
                            match key {
                                Key::Esc => should_quit = true,
                                Key::Char('\t') => {
                                    if left_panel.is_active() {
                                        left_panel.deactivate();
                                        right_panel.activate();
                                    } else {
                                        left_panel.activate();
                                        right_panel.deactivate();
                                    }
                                    active_panel.switch()
                                }
                                Key::F(5) => {
                                    let (source, destination) = match active_panel {
                                        ActivePanel::Left => {
                                            if let Some(selected_file) =
                                                left_panel.get_selected_file()
                                            {
                                                (selected_file, right_panel.pwd().to_path_buf())
                                            } else {
                                                // TODO: show error message about no selection
                                                (
                                                    left_panel.pwd().to_path_buf(),
                                                    right_panel.pwd().to_path_buf(),
                                                )
                                            }
                                        }
                                        ActivePanel::Right => {
                                            if let Some(selected_file) =
                                                right_panel.get_selected_file()
                                            {
                                                (selected_file, left_panel.pwd().to_path_buf())
                                            } else {
                                                // TODO: show error message about no selection
                                                (
                                                    right_panel.pwd().to_path_buf(),
                                                    left_panel.pwd().to_path_buf(),
                                                )
                                            }
                                        }
                                    };
                                    self.dialog = Some(Dialog::CopyDialog(CopyDialog::new(
                                        source.to_path_buf(),
                                        destination.to_path_buf(),
                                    )));
                                    self.focused_widget = Widgets::Dialog;
                                    self.input_mode = InputMode::Editing;
                                }
                                //Bottom Menu
                                Key::F(7) => {
                                    let path = match active_panel {
                                        ActivePanel::Left => left_panel.pwd(),
                                        ActivePanel::Right => right_panel.pwd(),
                                    };
                                    self.dialog = Some(Dialog::MkDirDialog(MkDirDialog::new(
                                        path.to_path_buf(),
                                    )));
                                    self.input_mode = InputMode::Editing;
                                    self.focused_widget = Widgets::Dialog;
                                }
                                Key::F(8) => {
                                    /*let selection = match active_panel {
                                        ActivePanel::Left => left_panel.get_selection(),
                                        ActivePanel::Right => right_panel.get_selection(),
                                    };*/
                                    //if let Some(idx) = selection {
                                    let path = match active_panel {
                                        ActivePanel::Left => left_panel.get_selected_file(),
                                        ActivePanel::Right => right_panel.get_selected_file(),
                                    };
                                    if let Some(path) = path {
                                        self.dialog =
                                            Some(Dialog::RmDirDialog(RmDirDialog::new(vec![path])));
                                        self.input_mode = InputMode::Editing;
                                        self.focused_widget = Widgets::Dialog;
                                    }
                                    //}
                                    /*
                                    if !selection.is_none() {
                                        let path = self.dialog = Some(Dialog::RmDirDialog(
                                            RmDirDialog::new(Vec::new(selection)),
                                        ));
                                        self.input_mode = InputMode::Editing;
                                        self.focused_widget = Widgets::Dialog;
                                    }*/
                                }
                                Key::F(9) => menu.select_next(),
                                Key::Left => {
                                    if menu.has_selection() {
                                        menu.select_previous()
                                    }
                                }
                                Key::Right => {
                                    if menu.has_selection() {
                                        menu.select_next()
                                    }
                                }
                                Key::Home => match active_panel {
                                    ActivePanel::Left => left_panel.select_first(),
                                    ActivePanel::Right => right_panel.select_first(),
                                },
                                Key::End => match active_panel {
                                    ActivePanel::Left => left_panel.select_last(),
                                    ActivePanel::Right => right_panel.select_last(),
                                },
                                Key::Up => match active_panel {
                                    ActivePanel::Left => left_panel.select_previous(),
                                    ActivePanel::Right => right_panel.select_previous(),
                                },
                                Key::Down => match active_panel {
                                    ActivePanel::Left => left_panel.select_next(),
                                    ActivePanel::Right => right_panel.select_next(),
                                },
                                Key::Char('\n') => match active_panel {
                                    ActivePanel::Left => left_panel.change_dir(),
                                    ActivePanel::Right => right_panel.change_dir(),
                                },
                                _ => {}
                            }
                        }
                        InputMode::Editing => {
                            if let Some(dialog) = &mut self.dialog {
                                match dialog {
                                    Dialog::CopyDialog(copy_dialog) => match key {
                                        Key::Char('\n') => {
                                            copy_dialog.handle_key(key);
                                        }
                                        Key::Esc => {
                                            self.input_mode = InputMode::Normal;
                                            self.dialog = None;
                                            self.focused_widget = Widgets::TwinPanel;
                                        }
                                        _ => {
                                            copy_dialog.handle_key(key);
                                        }
                                    },
                                    Dialog::MkDirDialog(mkdir_dialog) => match key {
                                        Key::Char('\n') => match mkdir_dialog.state() {
                                            crate::ui::MkDirDialogState::WaitingForInput => {
                                                let result = mkdir_dialog.create_dir();
                                                if let Ok(_) = result {
                                                    self.input_mode = InputMode::Normal;
                                                    self.dialog = None;
                                                    self.focused_widget = Widgets::TwinPanel;
                                                }
                                            }
                                            crate::ui::MkDirDialogState::DisplayErrorMessage(_) => {
                                                self.input_mode = InputMode::Normal;
                                                self.dialog = None;
                                                self.focused_widget = Widgets::TwinPanel;
                                            }
                                            crate::ui::MkDirDialogState::DirCreated => {
                                                self.input_mode = InputMode::Normal;
                                                self.dialog = None;
                                                self.focused_widget = Widgets::TwinPanel;
                                            }
                                        },
                                        Key::Esc => {
                                            self.input_mode = InputMode::Normal;
                                            self.dialog = None;
                                            self.focused_widget = Widgets::TwinPanel;
                                        }
                                        _ => mkdir_dialog.handle_key(key),
                                    },
                                    Dialog::RmDirDialog(rmdir_dialog) => match key {
                                        Key::Esc => {
                                            self.input_mode = InputMode::Normal;
                                            self.dialog = None;
                                            self.focused_widget = Widgets::TwinPanel;
                                        }
                                        _ => rmdir_dialog.handle_keys(key),
                                    },
                                }
                            }
                        }
                    },
                    Event::Tick => {
                        if let Some(dialog) = &mut self.dialog {
                            match dialog {
                                Dialog::CopyDialog(copy_dialog) => {
                                    copy_dialog.tick();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
