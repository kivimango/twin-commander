use crate::{
    app::{TuiRealmApplication, UserInterfaces},
    ui::{PanelMessage, PanelState, SELECT_ITEM},
};
use tuirealm::{
    props::{PropPayload, PropValue},
    AttrValue, Attribute, State, StateValue,
};

pub struct PanelMessageHandler {}

impl PanelMessageHandler {
    pub fn new() -> Self {
        PanelMessageHandler {}
    }

    pub fn handle_msg(
        &self,
        app: &mut TuiRealmApplication,
        panel_state: &mut PanelState,
        panel: &UserInterfaces,
        message: &PanelMessage,
    ) {
        match message {
            PanelMessage::SelectItem => {
                if let Ok(state) = app.state(panel) {
                    select_item(state, app, panel_state, panel);
                }
            }
            _ => {}
        }
    }
}

fn select_item(
    state: State,
    app: &mut TuiRealmApplication,
    panel_state: &mut PanelState,
    panel: &UserInterfaces,
) {
    match state {
        State::One(StateValue::Usize(cursor_pos)) => {
            let count = panel_state.files().len();

            // Possible crash: if the file count is 0, but there is always a parent dir at index 0 except when the user is at the root folder,
            // and it has no directories, which is unlikely
            if cursor_pos < count - 1 {
                panel_state.select(cursor_pos);
                let new_cursor_pos = cursor_pos + 1;
                let new_value =
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(new_cursor_pos)));
                let current_value =
                    AttrValue::Payload(PropPayload::One(PropValue::Usize(cursor_pos)));
                // mark the item at current cursor pos as selected
                let _ = app.attr(panel, Attribute::Custom(&SELECT_ITEM), current_value);
                // advance the cursos pos by one
                let _ = app.attr(panel, Attribute::Value, new_value);
            }
        }
        _ => {}
    }
}
