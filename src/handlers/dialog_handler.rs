use crate::{
    app::{ApplicationMessage, ApplicationModel, TuiRealmApplication},
    ui::DialogMessage,
};

pub struct DialogMessageHandler {}

impl DialogMessageHandler {
    pub fn new() -> Self {
        DialogMessageHandler {}
    }

    pub fn handle_msg(
        &self,
        app: &mut ApplicationModel,
        message: DialogMessage,
    ) -> Option<ApplicationMessage> {
        None
    }
}
