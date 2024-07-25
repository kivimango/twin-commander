use tuirealm::tui::layout::Constraint;
use tuirealm::tui::layout::Direction;
use tuirealm::tui::layout::Layout;
use tuirealm::tui::layout::Rect;
use tuirealm::State;

mod bottom_menu;
mod dialog;
mod menu;
mod panel;
mod panel_state;
mod table;
//mod user_interface;
//mod widgets;

use crate::core::sort::TableSortDirection;
use crate::core::sort::TableSortPredicate;

pub use self::bottom_menu::*;
pub use self::dialog::*;
pub use self::menu::*;
pub use self::panel::*;
pub use self::panel_state::*;
//pub use self::user_interface::UserInterface;
//pub use self::widgets::*;

#[derive(Debug, PartialEq)]
pub enum PanelMessage {
    ChangeSortDirection(TableSortDirection),
    ChangeDirectory(State),
    ChangeSortPredicate(TableSortPredicate),
    GoBackUp,
    SwitchPanel,
}

/// A list of available widgets to use in a `Panel`.
#[allow(unused)]
pub enum Widgets {
    /// Displays the content of the working directory in a table-like format.
    Table,
    /// Displays the contents of a text file.
    TextFileViewer,
}

/// Helper function to create a centered rect with a fixed height
/// and using up certain percentage of the available of width of `r`.
pub fn fixed_height_centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let percent_y = ((height as f64 / r.height as f64) * 100.0) as u16;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod test {
    use super::fixed_height_centered_rect;
    use tuirealm::tui::layout::Rect;

    #[test]
    fn test_fixed_height_centered_rect() {
        let percent_x = 50;
        let height = 10;
        let available_area = Rect::new(0, 0, 100, 100);
        let area = fixed_height_centered_rect(percent_x, height, available_area);

        assert_eq!(area.x, 25);
        assert_eq!(area.y, 45);
        assert_eq!(area.width, 50);
        assert_eq!(area.height, height);
    }
}
