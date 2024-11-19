use std::path::PathBuf;

use crate::{
    app::{ApplicationMessage, ApplicationModel},
    ui::{fixed_height_centered_rect, DialogMessage, ErrorDialog, RemoveConfirmationDialog},
};

pub fn handle_msg(
    app: &mut ApplicationModel,
    message: DialogMessage,
) -> Option<ApplicationMessage> {
    match message {
        DialogMessage::ShowRmDialog => {
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
                        let rm_dialog =
                            Box::new(RemoveConfirmationDialog::new().with_files(vec![path]));
                        app.show_dialog(rm_dialog, rm_dialog_area);
                    }
                // the cursor is at the parent directory
                } else {
                    let error_dialog_area = fixed_height_centered_rect(33, 5, app.area());
                    let error_dialog = Box::new(
                        ErrorDialog::new()
                            .with_title("Error")
                            .with_message(
                                "The operation is not possible with the parent directory!",
                            )
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
        _ => Some(ApplicationMessage::None),
    }
}
