use std::path::PathBuf;

use tuirealm::State;

use crate::{
    app::{ApplicationMessage, ApplicationModel, UserInterfaces},
    ui::{
        fixed_height_centered_rect, DialogMessage, ErrorDialog, HelpDialog, MkDirDialog,
        PanelOpionsDialog, RemoveConfirmationDialog, SortingDialog, TransferConfirmationDialog,
        TransferProgressDialog,
    },
};

pub fn handle_msg(
    app: &mut ApplicationModel,
    message: DialogMessage,
) -> Option<ApplicationMessage> {
    match message {
        DialogMessage::CloseDialog => close_dialog(app),
        DialogMessage::ShowHelpDialog => show_help_dialog(app),
        DialogMessage::ShowMoveDialog => show_transfer_dialog(app, false),
        DialogMessage::ShowCopyDialog => show_transfer_dialog(app, true),
        DialogMessage::ShowMkDirDialog => show_mkdir_dialog(app),
        DialogMessage::ShowRmDialog => show_rm_dialog(app),
        DialogMessage::ShowSortDialog => show_sort_dialog(app),
        DialogMessage::ShowPanelOptionsDialog => show_panel_options_dialog(app),
        DialogMessage::ShowFilterDialog => None,
        DialogMessage::BeginTransfer(delete_source) => begin_transfer(app, delete_source),
        DialogMessage::RemoveSelectedFiles => Some(ApplicationMessage::None),
        DialogMessage::CreateDirectory(state) => create_dir(app, state),
    }
}

fn show_help_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    let area = fixed_height_centered_rect(50, 14, app.area());
    let help_dialog = Box::new(HelpDialog::new());
    app.show_dialog(help_dialog, area);
    Some(ApplicationMessage::None)
}

fn show_transfer_dialog(
    app: &mut ApplicationModel,
    keep_source: bool,
) -> Option<ApplicationMessage> {
    let title = match keep_source {
        false => "Move",
        true => "Copy",
    };
    let (source, target) = app.get_pwds();
    let area = fixed_height_centered_rect(50, 8, app.area());
    let move_dialog = TransferConfirmationDialog::default()
        .source(source)
        .target(target)
        .keep_source(keep_source)
        .title(title);
    let move_dialog = Box::new(move_dialog);
    app.show_dialog(move_dialog, area);
    Some(ApplicationMessage::None)
}

fn show_mkdir_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    let area = fixed_height_centered_rect(50, 9, app.area());
    let mkdir_dialog = Box::new(MkDirDialog::new());
    app.show_dialog(mkdir_dialog, area);
    Some(ApplicationMessage::None)
}

fn show_rm_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    let selected_files_count = app.panel_states().selected_files_count();

    // if no items are marked as selected, use the file at the currently active panel's cursor
    // as a parameter of the RemoveConfirmationDialog
    if selected_files_count == 0 {
        let widget = app.tui_realm().focus().unwrap();
        let cursor_pos = app
            .tui_realm()
            .state(widget)
            .unwrap()
            .unwrap_one()
            .unwrap_usize();

        if cursor_pos != 0 {
            if let Some(file_at_cursor) = app.panel_states().get_file(cursor_pos) {
                let mut path = PathBuf::from(app.panel_states().pwd());
                path.push(&file_at_cursor.name);
                let rm_dialog_area = fixed_height_centered_rect(50, 5, app.area());
                let rm_dialog = Box::new(RemoveConfirmationDialog::new().with_files(vec![path]));
                app.show_dialog(rm_dialog, rm_dialog_area);
            }
        // the cursor is at the parent directory
        } else {
            let error_dialog_area = fixed_height_centered_rect(33, 5, app.area());
            let error_dialog = Box::new(
                ErrorDialog::new()
                    .with_title("Error")
                    .with_message("The operation is not possible with the parent directory!")
                    .with_button_title("   OK   "),
            );
            app.show_dialog(error_dialog, error_dialog_area);
        }
    } else {
        let mut paths = Vec::with_capacity(selected_files_count);
        let selected_indices = app.panel_states().selected_files();

        for index in selected_indices {
            if let Some(f) = app.panel_states().get_file(index) {
                let mut path = PathBuf::from(app.panel_states().pwd());
                path.push(&f.name);

                paths.push(path);
            }
        }

        let rm_dialog_area = fixed_height_centered_rect(50, 5, app.area());
        let rm_dialog = Box::new(RemoveConfirmationDialog::new().with_files(paths));
        app.show_dialog(rm_dialog, rm_dialog_area);
    }

    Some(ApplicationMessage::None)
}

fn show_sort_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    let area = fixed_height_centered_rect(50, 9, app.area());
    let predicate = app.panel_states().sort_predicate();
    let direction = app.panel_states().sort_direction();
    let sort_dialog = Box::new(SortingDialog::new(predicate, direction));
    app.show_dialog(sort_dialog, area);
    Some(ApplicationMessage::None)
}

fn show_panel_options_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    let area = fixed_height_centered_rect(50, 9, app.area());
    let config = app.get_config();
    let panel_options_dialoge = Box::new(PanelOpionsDialog::new(config));
    app.show_dialog(panel_options_dialoge, area);
    Some(ApplicationMessage::None)
}

fn begin_transfer(app: &mut ApplicationModel, keep_source: bool) -> Option<ApplicationMessage> {
    let is_dialog_mounted = app.tui_realm().mounted(&UserInterfaces::Dialog);
    if is_dialog_mounted {
        app.tui_realm_mut().umount(&UserInterfaces::Dialog).unwrap();
        let area = fixed_height_centered_rect(50, 9, app.area());
        let title = match keep_source {
            false => "Copy",
            true => "Move",
        };
        let (source, target) = app.get_pwds();
        let dialog = TransferProgressDialog::new()
            .source(source)
            .target(target)
            .title(title);
        //dialog.attr(Attribute::Content, AttrValue::String(String::from("x.txt")));
        let progress_dialog = Box::new(dialog);
        app.show_dialog(progress_dialog, area);
        app.tui_realm_mut().active(&UserInterfaces::Dialog).unwrap();
    }

    Some(ApplicationMessage::None)
}

fn create_dir(app: &mut ApplicationModel, state: State) -> Option<ApplicationMessage> {
    let mut current_dir = PathBuf::from(app.panel_states().pwd());
    let new_dir_name = state.unwrap_one().unwrap_string();
    current_dir.push(new_dir_name);
    let path = current_dir.to_owned();

    match std::fs::create_dir(&path) {
        Ok(_) => {
            return Some(ApplicationMessage::Dialog(DialogMessage::CloseDialog));
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

fn close_dialog(app: &mut ApplicationModel) -> Option<ApplicationMessage> {
    app.close_dialog();
    Some(ApplicationMessage::None)
}
