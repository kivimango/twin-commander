use tui::layout::{Constraint, Direction, Layout, Rect};

mod bottom_menu;
mod dialog;
mod menu;
mod table;

pub use self::bottom_menu::*;
pub use self::dialog::*;
pub use self::menu::*;
pub use self::table::*;

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
    use tui::layout::Rect;

    #[test]
    fn test_fixed_height_centered_rect() {
        let percent_x = 50;
        let height = 10;
        let available_area = Rect::new(0, 0, 100, 100);
        let area = fixed_height_centered_rect(percent_x, height, available_area);

        println!("{:?}", area);

        assert_eq!(area.x, 25);
        assert_eq!(area.y, 45);
        assert_eq!(area.width, 50);
        assert_eq!(area.height, height);
    }
}
