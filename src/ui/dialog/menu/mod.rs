mod top;

pub use self::top::*;
use crate::app::Application;
use crate::core::config::Configuration;
use crate::ui::user_interface::ActivePanel;
use std::io::Stdout;
use termion::event::Key;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::Frame;

/// Common top menu dialog functionality encapsulated into this trait.
/// The Boxed in the typename refers to that implementors this trait will be Box-ed
/// when UserInterface creates them.
pub trait BoxedDialog {
    /// Allows the dialog to modify the current application configuration.
    /// The dialog should only modify configuration values that are rendering.
    /// The changes made to the configuration resides only in memory,
    /// until the application is requested close.
    fn change_configuration(&mut self, config: &mut Configuration, activa_panel: ActivePanel);

    /// Key handling logic for the dialog implementing this trait.
    /// UserInterfacee will pass keys only if `InputMode` is `Editing`
    /// and the dialog is open (shown).
    fn handle_keys(&mut self, key: Key, app: &mut Application);

    /// Renders the current state of the dialog into the current `Frame` for the given `Area`.
    /// The full area of the screen is available to use for rendering.
    /// Dialogs should render itselfs to the center of the screen.
    fn render(&self, area: Rect, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>);

    /// Notifies UserInterface that the dialog implementing this trait is requesting chagning
    /// the current application configuration.
    /// On the next tick event, the UserInterface will call the change_configuration() method.
    fn request_config_change(&self) -> bool;

    /// Notifies UserInterface that the dialog implementing this trait is requesting closing
    /// itself, and the UserInterface should not draw this dialog anymore on the screen.
    fn should_quit(&self) -> bool;
}
