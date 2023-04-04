mod panel;

pub use self::panel::*;

/// A list of available widgets to use in a `Panel`.
pub enum Widgets {
    /// Displays the content of the working directory in a table-like format.
    Table,
    /// Displays the contents of a text file.
    TextFileViewer
}
