use crate::core::config::{self, try_load_from_file, try_save_to_file, Configuration};
use crate::core::list_dir::{DirContent, FilterOptions};
use crate::ui::{
    fixed_height_centered_rect, Dialog, DialogMessage, HelpDialog, MkDirDialog, PanelState,
};
use crate::ui::{
    BottomMenu, PanelMessage, TableSortDirection, TableSortPredicate, TableView, TopMenu,
    TopMenuMessage,
};
use humansize::{SizeFormatter, DECIMAL};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tuirealm::application::ApplicationResult;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    Alignment, Color, PropPayload, PropValue, Style, Table, TableBuilder, TextSpan,
};
use tuirealm::terminal::TerminalBridge;
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::Clear;
use tuirealm::{
    Application, AttrValue, Attribute, EventListenerCfg, NoUserEvent, PollStrategy, State,
    StateValue, Sub, SubClause, SubEventClause, Update,
};

type TuiRealmApplication = Application<UserInterfaces, ApplicationMessage, NoUserEvent>;

const LEFT_PANEL_IDX: usize = 0;
const RIGHT_PANEL_IDX: usize = 1;

/// List of available user interface components in the application.
/// Variants are uniqe identifiers of those components used by tuirealm.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum UserInterfaces {
    Dialog,
    Topmenu,
    LeftPanel,
    RightPanel,
    BottomMenu,
}

/// List of available messages the application can handle.
#[derive(Debug, PartialEq)]
pub enum ApplicationMessage {
    /// Requests closing the application
    Close,

    Dialog(DialogMessage),

    /// Brings up the top menu by stealing the focus from the currently focused component
    FocusBottomMenu,

    /// Messages sent by panels
    Panel(PanelMessage),

    TopMenu(TopMenuMessage),

    /// Indicates that the component in current focus has handled its changes internally,
    /// and it wont send an application message, but the ui should be redrawn regardless
    None,
}

pub struct ApplicationModel {
    app: TuiRealmApplication,
    active_panel: usize,
    area: Rect,
    dialog: Option<Dialog>,
    should_quit: bool,
    redraw: bool,
    panel_states: [PanelState; 2],
}

impl ApplicationModel {
    pub fn new() -> Self {
        ApplicationModel {
            app: initialize(),
            active_panel: LEFT_PANEL_IDX,
            area: Rect::default(),
            dialog: None,
            should_quit: false,
            redraw: true,
            panel_states: [PanelState::new(), PanelState::new()],
        }
    }

    /// Initializes the left and right panels by reading their distinct configuration.
    /// Must be called after `self.mount_views()`.
    fn init_panels(&mut self, config: &Configuration) {
        let left_path = config.left_table_config().path();
        let left_sort_direction =
            TableSortDirection::from(config.left_table_config().sort_direction());
        let left_sort_predicate =
            TableSortPredicate::from(config.left_table_config().sort_predicate());
        let left_filters = FilterOptions {
            show_hidden_files: config.show_hidden_files(),
        };
        self.panel_states[LEFT_PANEL_IDX].set_current_path(left_path);
        self.panel_states[LEFT_PANEL_IDX].set_direction(left_sort_direction);
        self.panel_states[LEFT_PANEL_IDX].set_predicate(left_sort_predicate);
        self.panel_states[LEFT_PANEL_IDX].set_filters(left_filters);
        let left = self.panel_states[LEFT_PANEL_IDX]
            .list_files(left_path)
            .unwrap();
        self.panel_states[LEFT_PANEL_IDX].sort();
        self.panel_states[LEFT_PANEL_IDX].set_files(left);
        self.set_panel_headers(
            &UserInterfaces::LeftPanel,
            self.panel_states[LEFT_PANEL_IDX].headers(),
        )
        .unwrap();

        let right_path = config.right_table_config().path();
        let right_sort_direction =
            TableSortDirection::from(config.right_table_config().sort_direction());
        let right_sort_predicate =
            TableSortPredicate::from(config.right_table_config().sort_predicate());
        let right_filters = FilterOptions {
            show_hidden_files: config.show_hidden_files(),
        };
        self.panel_states[RIGHT_PANEL_IDX].set_current_path(right_path);
        self.panel_states[RIGHT_PANEL_IDX].set_direction(right_sort_direction);
        self.panel_states[RIGHT_PANEL_IDX].set_predicate(right_sort_predicate);
        self.panel_states[RIGHT_PANEL_IDX].set_filters(right_filters);
        let right = self.panel_states[RIGHT_PANEL_IDX]
            .list_files(right_path)
            .unwrap();
        self.panel_states[RIGHT_PANEL_IDX].sort();
        self.panel_states[RIGHT_PANEL_IDX].set_files(right);
        self.set_panel_headers(
            &UserInterfaces::RightPanel,
            self.panel_states[RIGHT_PANEL_IDX].headers(),
        )
        .unwrap();

        self.app
            .attr(
                &UserInterfaces::LeftPanel,
                Attribute::Content,
                AttrValue::Table(files_to_table(self.panel_states[LEFT_PANEL_IDX].files())),
            )
            .unwrap();
        self.set_panel_title(&UserInterfaces::LeftPanel, left_path.display().to_string())
            .unwrap();
        self.app
            .attr(
                &UserInterfaces::RightPanel,
                Attribute::Content,
                AttrValue::Table(files_to_table(self.panel_states[RIGHT_PANEL_IDX].files())),
            )
            .unwrap();
        self.set_panel_title(
            &UserInterfaces::RightPanel,
            right_path.display().to_string(),
        )
        .unwrap();
    }

    fn mount_views(&mut self) {
        let top_menu = TopMenu::new()
            .background(Color::Cyan)
            .foreground(Color::Black)
            .item_style(Style::default().bg(Color::Cyan).fg(Color::White))
            .selected_item_style(Style::default().bg(Color::Black).fg(Color::White));
        let bottom_menu = BottomMenu::new()
            .background(Color::Cyan)
            .label_foreground(Color::Black)
            .function_key_background(Color::Black)
            .function_key_foreground(Color::White);
        let left_table = TableView::new();
        let right_table = TableView::new();

        self.app
            .mount(
                UserInterfaces::Topmenu,
                Box::new(top_menu),
                vec![
                    Sub::new(
                        SubEventClause::Any,
                        SubClause::HasAttrValue(
                            UserInterfaces::Topmenu,
                            Attribute::Focus,
                            AttrValue::Flag(true),
                        ),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            modifiers: KeyModifiers::NONE,
                            code: Key::Function(9),
                        }),
                        SubClause::Always,
                    ),
                ],
            )
            .expect("Failed to mount top menu component into the view!");
        self.app
            .mount(UserInterfaces::LeftPanel, Box::new(left_table), vec![])
            .expect("Failed to mount left tableview component into the view!");
        self.app
            .mount(UserInterfaces::RightPanel, Box::new(right_table), vec![])
            .expect("Failed to mount right tableview component into the view!");
        self.app
            .mount(UserInterfaces::BottomMenu, Box::new(bottom_menu), vec![])
            .expect("Failed to mount bottom menu component into the view!");
        self.app
            .active(&UserInterfaces::LeftPanel)
            .expect("Failed to activate bottom menu component!");
    }

    // Runs the main event loop for the application, handling user input and updating the user interface accordingly.
    ///
    /// # Arguments
    ///
    /// * `terminal` - A mutable reference to the terminal adapter instance used by the application.
    ///
    /// # Remarks
    ///
    /// This method continuously listens for user input and updates the user interface based on the current input mode.
    /// It also handles events such as quitting the application and saving the configuration before exiting.
    /// Performs initial checks for the configuration file and its path before starting the event loop.
    /// If the configuration file is not found, the application attempts to re-create it.
    /// Subsequently, the configuration data is loaded from the configuration file.
    pub fn run(&mut self, terminal: &mut TerminalBridge) {
        let mut config = get_config();
        self.area = terminal.raw_mut().get_frame().size();
        self.mount_views();
        self.init_panels(&config);

        while !self.should_quit {
            match self.app.tick(PollStrategy::Once) {
                Ok(messages) => {
                    self.redraw = true;
                    for message in messages {
                        let mut msg = Some(message);
                        while msg.is_some() {
                            msg = self.update(msg);
                        }
                    }
                }
                Err(error) => eprintln!("Error during tick: {error}"),
            }

            if self.redraw {
                self.view(terminal);
                self.redraw = false;
            }
        }

        self.sync_config(&mut config);
        save_config(&config);
    }

    fn view(&mut self, terminal: &mut TerminalBridge) {
        if let Err(error) = terminal.raw_mut().draw(|frame| {
            let frame_size = frame.size();
            let layout = Layout::default()
                .constraints([
                    Constraint::Min(1),
                    Constraint::Percentage(95),
                    Constraint::Min(1),
                ])
                .direction(Direction::Vertical)
                .split(frame_size);

            let table_layout = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(Direction::Horizontal)
                .split(layout[1]);

            self.app
                .view(&UserInterfaces::LeftPanel, frame, table_layout[0]);
            self.app
                .view(&UserInterfaces::RightPanel, frame, table_layout[1]);
            self.app.view(&UserInterfaces::BottomMenu, frame, layout[2]);

            // Draw menu after the panels to able to show expanded menus over content
            self.app.view(&UserInterfaces::Topmenu, frame, layout[0]);

            // Render popup at last over everything
            if self.app.mounted(&UserInterfaces::Dialog) {
                if let Some(dialog) = &self.dialog {
                    let popup_area = dialog.area;
                    frame.render_widget(Clear, popup_area);
                    self.app.view(&UserInterfaces::Dialog, frame, popup_area);
                }
            }
        }) {
            eprint!("Error during drawing frame: {error}");
        }
    }

    fn select_file(&mut self, component: &UserInterfaces, idx: usize) -> ApplicationResult<()> {
        self.app.attr(
            component,
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(idx))),
        )
    }

    fn select_first(&mut self, component: &UserInterfaces) -> ApplicationResult<()> {
        self.app.attr(
            component,
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(0))),
        )
    }

    fn select_parent_or_first(
        &mut self,
        panel: &UserInterfaces,
        active_panel: usize,
        parent_dir: &Path,
    ) {
        let file_count = self.panel_states[active_panel].files().len();
        if file_count == 1 {
            self.select_first(panel).unwrap();
        } else if let Some(parent_dir_name) = parent_dir.file_name() {
            if let Some(parent_dir_idx) = self.panel_states[active_panel]
                .files()
                .iter()
                .filter(|f| f.is_dir)
                .position(|f| f.name.as_str().eq(parent_dir_name))
            {
                if parent_dir_idx <= file_count {
                    self.select_file(panel, parent_dir_idx).unwrap();
                }
            } else {
                self.select_first(panel).unwrap();
            }
        }
    }

    fn set_panel_files(&mut self, panel: &UserInterfaces) -> ApplicationResult<()> {
        let focused_panel = active_panel_idx(panel).unwrap();
        let files = self.panel_states[focused_panel].files();
        let table = files_to_table(files);
        self.app
            .attr(panel, Attribute::Content, AttrValue::Table(table))
    }

    fn set_panel_headers(
        &mut self,
        panel: &UserInterfaces,
        headers: Vec<String>,
    ) -> ApplicationResult<()> {
        self.app.attr(
            panel,
            Attribute::Text,
            AttrValue::Payload(PropPayload::Vec(
                headers
                    .iter()
                    .map(|h| PropValue::Str(h.to_string()))
                    .collect(),
            )),
        )
    }

    fn set_panel_title(&mut self, panel: &UserInterfaces, title: String) -> ApplicationResult<()> {
        self.app.attr(
            panel,
            Attribute::Title,
            AttrValue::Title((title, Alignment::Left)),
        )
    }

    fn sync_config(&self, config: &mut Configuration) {
        config.set_show_hidden_files(
            self.panel_states[LEFT_PANEL_IDX]
                .filters()
                .show_hidden_files,
        );
        config
            .left_table_config_mut()
            .set_path(self.panel_states[LEFT_PANEL_IDX].pwd().to_owned());
        config
            .left_table_config_mut()
            .set_predicate(self.panel_states[LEFT_PANEL_IDX].sort_predicate().into());
        config
            .left_table_config_mut()
            .set_sort_direction(self.panel_states[LEFT_PANEL_IDX].sort_direction().into());
        config
            .right_table_config_mut()
            .set_path(self.panel_states[RIGHT_PANEL_IDX].pwd().to_owned());
        config
            .right_table_config_mut()
            .set_predicate(self.panel_states[RIGHT_PANEL_IDX].sort_predicate().into());
        config
            .right_table_config_mut()
            .set_sort_direction(self.panel_states[RIGHT_PANEL_IDX].sort_direction().into());
    }
}

impl Update<ApplicationMessage> for ApplicationModel {
    fn update(&mut self, msg: Option<ApplicationMessage>) -> Option<ApplicationMessage> {
        if let Some(message) = msg {
            self.redraw = true;

            return match message {
                ApplicationMessage::Close => {
                    self.should_quit = true;
                    None
                }
                ApplicationMessage::Dialog(dialog_message) => match dialog_message {
                    DialogMessage::ShowHelpDialog => {
                        let help_dialog = Box::new(HelpDialog::new());
                        self.dialog = Some(Dialog {
                            area: fixed_height_centered_rect(50, 14, self.area),
                        });
                        self.app
                            .mount(UserInterfaces::Dialog, help_dialog, vec![])
                            .unwrap();
                        self.app.active(&UserInterfaces::Dialog).unwrap();
                        Some(ApplicationMessage::None)
                    }
                    DialogMessage::ShowMkDirDialog => {
                        let mkdir_dialog = Box::new(MkDirDialog::new());
                        self.dialog = Some(Dialog {
                            area: fixed_height_centered_rect(50, 7, self.area),
                        });
                        self.app
                            .mount(UserInterfaces::Dialog, mkdir_dialog, vec![])
                            .unwrap();
                        self.app.active(&UserInterfaces::Dialog).unwrap();
                        Some(ApplicationMessage::None)
                    }
                    DialogMessage::CreateDirectory(state) => {
                        let mut current_dir =
                            PathBuf::from(self.panel_states[self.active_panel].pwd());
                        let new_dir_name = state.unwrap_one().unwrap_string();
                        current_dir.push(new_dir_name);
                        let path = current_dir.to_owned();

                        match std::fs::create_dir(&path) {
                            Ok(_) => {
                                println!("ok");
                                return Some(ApplicationMessage::Dialog(
                                    DialogMessage::CloseDialog,
                                ));
                            }
                            Err(error) => {
                                // TODO: display error message
                                eprintln!(
                                    "error creating new directory at {} : {} ",
                                    path.display(),
                                    error
                                );
                            }
                        }

                        Some(ApplicationMessage::None)
                    }
                    DialogMessage::CloseDialog => {
                        if self.app.mounted(&UserInterfaces::Dialog) {
                            self.dialog = None;
                            self.app.umount(&UserInterfaces::Dialog).unwrap();
                        }
                        Some(ApplicationMessage::None)
                    }
                },
                ApplicationMessage::FocusBottomMenu => {
                    self.app.active(&UserInterfaces::BottomMenu).unwrap();
                    Some(ApplicationMessage::None)
                }
                ApplicationMessage::Panel(panel_msg) => match panel_msg {
                    PanelMessage::ChangeSortDirection(direction) => {
                        if let Some(component_id) = self.app.focus().cloned() {
                            if let Some(active_panel) = active_panel_idx(&component_id) {
                                self.panel_states[active_panel].set_direction(direction);
                                self.set_panel_files(&component_id).unwrap();
                                let headers = self.panel_states[active_panel].headers();
                                self.set_panel_headers(&component_id, headers).unwrap();
                            }
                        }
                        Some(ApplicationMessage::None)
                    }
                    PanelMessage::ChangeSortPredicate(predicate) => {
                        if let Some(component_id) = self.app.focus().cloned() {
                            if let Some(active_panel) = active_panel_idx(&component_id) {
                                self.panel_states[active_panel].set_predicate(predicate);
                                self.set_panel_files(&component_id).unwrap();
                                let headers = self.panel_states[active_panel].headers();
                                self.set_panel_headers(&component_id, headers).unwrap();
                            }
                        }
                        Some(ApplicationMessage::None)
                    }
                    PanelMessage::ChangeDirectory(state) => match state {
                        State::One(StateValue::Usize(index)) => {
                            if let Some(component_id) = self.app.focus().cloned() {
                                if let Some(active_panel) = active_panel_idx(&component_id) {
                                    let parent_dir =
                                        PathBuf::from(self.panel_states[active_panel].pwd());
                                    match self.panel_states[active_panel].cd(index) {
                                        Ok(_) => {
                                            self.set_panel_files(&component_id).unwrap();
                                            self.set_panel_title(
                                                &component_id,
                                                self.panel_states[active_panel]
                                                    .pwd()
                                                    .display()
                                                    .to_string(),
                                            )
                                            .unwrap();

                                            if index == 0 {
                                                self.select_parent_or_first(
                                                    &component_id,
                                                    active_panel,
                                                    &parent_dir,
                                                );
                                            } else {
                                                self.select_first(&component_id).unwrap();
                                            }
                                            return Some(ApplicationMessage::None);
                                        }
                                        Err(_error) => {
                                            // show error message in popup
                                            return None;
                                        }
                                    }
                                }
                            }
                            // TODO: show error message
                            return None;
                        }
                        _ => None,
                    },
                    PanelMessage::GoBackUp => {
                        if let Some(component_id) = self.app.focus().cloned() {
                            if let Some(active_panel) = active_panel_idx(&component_id) {
                                let parent_dir =
                                    PathBuf::from(self.panel_states[active_panel].pwd());

                                match self.panel_states[active_panel].cd(0) {
                                    Ok(_) => {
                                        self.set_panel_files(&component_id).unwrap();
                                        self.set_panel_title(
                                            &component_id,
                                            self.panel_states[active_panel]
                                                .pwd()
                                                .display()
                                                .to_string(),
                                        )
                                        .unwrap();
                                        self.select_parent_or_first(
                                            &component_id,
                                            active_panel,
                                            &parent_dir,
                                        );
                                    }
                                    Err(_error) => {}
                                }
                            }
                        }
                        Some(ApplicationMessage::None)
                    }
                    PanelMessage::SwitchPanel => {
                        match self.active_panel {
                            LEFT_PANEL_IDX => self.active_panel = RIGHT_PANEL_IDX,
                            RIGHT_PANEL_IDX => self.active_panel = LEFT_PANEL_IDX,
                            _ => {}
                        }
                        if let Some(focused_component) = self.app.focus() {
                            match focused_component {
                                UserInterfaces::LeftPanel => {
                                    self.app.active(&UserInterfaces::RightPanel).unwrap()
                                }
                                UserInterfaces::RightPanel => {
                                    self.app.active(&UserInterfaces::LeftPanel).unwrap()
                                }
                                _ => {}
                            };
                        }
                        return Some(ApplicationMessage::None);
                    }
                },
                ApplicationMessage::TopMenu(top_menu_msg) => {
                    match top_menu_msg {
                        TopMenuMessage::Blur => {
                            self.app.active(&UserInterfaces::LeftPanel).unwrap();
                        }
                        TopMenuMessage::Focus => {
                            if let Some(focused_component) = self.app.focus() {
                                if !focused_component.eq(&UserInterfaces::Topmenu) {
                                    self.app.active(&UserInterfaces::Topmenu).unwrap();
                                }
                            }
                        }
                    }
                    Some(ApplicationMessage::None)
                }
                ApplicationMessage::None => None,
            };
        }
        None
    }
}

fn initialize() -> TuiRealmApplication {
    tuirealm::Application::init(
        EventListenerCfg::default()
            .default_input_listener(Duration::from_millis(50))
            .poll_timeout(Duration::from_millis(50))
            .tick_interval(Duration::from_millis(50)),
    )
}

fn files_to_table(files: &[DirContent]) -> Table {
    let mut builder = TableBuilder::default();
    let count = files.len();

    for (idx, f) in files.iter().enumerate() {
        builder.add_col(TextSpan::from(&f.name));
        let size = match f.size {
            Some(size) => format!("{}", SizeFormatter::new(size, DECIMAL)),
            None => "<DIR>".to_string(),
        };
        builder.add_col(size.into());
        builder.add_col(TextSpan::new(&f.date));

        if idx != count - 1 {
            builder.add_row();
        }
    }
    builder.build()
}

fn active_panel_idx(panel: &UserInterfaces) -> Option<usize> {
    match panel {
        UserInterfaces::LeftPanel => Some(LEFT_PANEL_IDX),
        UserInterfaces::RightPanel => Some(RIGHT_PANEL_IDX),
        _ => None,
    }
}

fn get_config() -> Configuration {
    let default_config = Configuration::default();

    if config::is_dir_exists() {
        if config::is_file_exists() {
            match try_load_from_file() {
                Ok(config) => config,
                Err(_) => default_config, // TODO: log error
            }
        } else {
            match config::try_save_to_file(&default_config) {
                Ok(_) => Configuration::default(),
                Err(_) => Configuration::default(),
            };
            default_config
        }
    } else if let Err(_error) = config::create_config_dir() {
        //TODO: log error
        default_config
    } else {
        match config::try_save_to_file(&default_config) {
            Ok(_) => default_config,
            Err(_) => default_config,
        }
    }
}

fn save_config(config: &Configuration) {
    if !config::is_dir_exists() {
        if config::create_config_dir().is_ok() {
            let _ = try_save_to_file(config);
        }
    } else {
        let _ = try_save_to_file(config);
    }
}
