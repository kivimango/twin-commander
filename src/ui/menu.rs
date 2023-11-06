use std::borrow::Cow;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{Block, StatefulWidget, Widget},
};

struct MenuItem {
    title: Cow<'static, str>,
    highlighted: bool,
}

struct SubMenu {
    label: Cow<'static, str>,
    items: Vec<MenuItem>,
    highlighted_item_idx: usize,
    selected: bool,
    /// Width of the longest item title
    width: usize,
}

impl SubMenu {
    fn new(label: impl Into<Cow<'static, str>>, items: Vec<MenuItem>) -> Self {
        let longest_width = items
            .iter()
            .max_by(|x, y| x.title.len().cmp(&y.title.len()))
            .unwrap()
            .title
            .len();
        SubMenu {
            label: label.into(),
            items,
            highlighted_item_idx: 0,
            selected: false,
            width: longest_width,
        }
    }

    pub(crate) fn select_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.highlighted_item_idx) {
            item.highlighted = true;
        }
    }

    pub(crate) fn deselect_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.highlighted_item_idx) {
            item.highlighted = false;
        }
    }

    pub(crate) fn select_first(&mut self) {
        self.deselect_current();
        self.highlighted_item_idx = 0;
        self.select_current();
    }
}

/// Represents the menubar starting from the upper left corner.
#[derive(Default)]
pub struct Menu<'block> {
    style: Style,
    item_style: Style,
    selected_item_style: Style,
    submenu_block: Option<Block<'block>>,
}

impl<'block> Menu<'block> {
    /// Sets the top menu bar's style.
    pub(crate) fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the menu item's style in the top bar.
    pub(crate) fn item_style(mut self, item_style: Style) -> Self {
        self.item_style = item_style;
        self
    }

    /// Sets the menu item's style when it is selected.
    /// This style is also used for the selected submenu item's style.
    pub(crate) fn selected_item_style(mut self, selected_item_style: Style) -> Self {
        self.selected_item_style = selected_item_style;
        self
    }

    /// Sets the block around the submenu group when it is expanded.
    pub(crate) fn submenu_block(mut self, block: Block<'block>) -> Self {
        self.submenu_block = Some(block);
        self
    }
}

/// Keeps track of the state of the Menu preserved between two draw calls.
pub struct MenuState {
    items: Vec<SubMenu>,
    selected_item_idx: usize,
}

impl MenuState {
    fn new(items: Vec<SubMenu>) -> Self {
        MenuState {
            items,
            selected_item_idx: 0,
        }
    }

    /// Activates the menu: selects the first menu item,
    /// then select the first submenu item in this group.
    pub(crate) fn activate(&mut self) {
        self.selected_item_idx = 0;
        self.select_current();
        self.items[0].select_first();
    }

    /// Deselects the currently selected menu item.
    /// In the inactive state, menu group dropdowns are not drawned.
    pub(crate) fn deactivate(&mut self) {
        self.deselect_current()
    }

    /// Selects the previous menu item (from right to left).
    /// Calling this method wont has no effect when the currently selected menu item is the first.
    pub(crate) fn select_previous(&mut self) {
        if self.selected_item_idx == 0 {
            return;
        }

        self.deselect_current();
        self.selected_item_idx -= 1;
        self.select_current();
        self.items[self.selected_item_idx].select_first();
    }

    /// Selects the next menu item (from left to right).
    /// Calling this method has no effect when the currently selected menu item is the last.
    pub(crate) fn select_next(&mut self) {
        if self.selected_item_idx == self.items.len() - 1 {
            return;
        }

        self.deselect_current();
        self.selected_item_idx += 1;
        self.select_current();
        self.items[self.selected_item_idx].select_first();
    }

    /// Selects the submenu item one upper than the currently one (from bottom to top).
    /// Calling this method has no effect when the currently selected menu item is the first (the highest).
    pub(crate) fn up(&mut self) {
        if let Some(submenu) = self.items.get_mut(self.selected_item_idx) {
            if let Some(_item) = submenu.items.get_mut(submenu.highlighted_item_idx) {
                if submenu.highlighted_item_idx == 0 {
                    return;
                }

                submenu.deselect_current();
                submenu.highlighted_item_idx -= 1;
                submenu.select_current();
            }
        }
    }

    /// Selects the submenu item one lower than the currently one (from top to bottom).
    /// Calling this method has no effect when the currently selected menu item is the last (the lowest).
    pub(crate) fn down(&mut self) {
        if let Some(submenu) = self.items.get_mut(self.selected_item_idx) {
            if let Some(_item) = submenu.items.get_mut(submenu.highlighted_item_idx) {
                if submenu.highlighted_item_idx == submenu.items.len() - 1 {
                    return;
                }

                submenu.deselect_current();
                submenu.highlighted_item_idx += 1;
                submenu.select_current();
            }
        }
    }

    fn select_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_item_idx) {
            item.selected = true;
        }
    }

    fn deselect_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_item_idx) {
            item.selected = false;
        }
    }

    pub(crate) fn selected_item(&self) -> usize {
        self.selected_item_idx
    }

    /// Creates a pre-made Menu instance.
    pub(crate) fn new_premade() -> Self {
        MenuState::new(vec![
            SubMenu::new(
                " Left ",
                vec![
                    MenuItem {
                        title: "Sort order".into(),
                        highlighted: false,
                    },
                    MenuItem {
                        title: "Filter".into(),
                        highlighted: false,
                    },
                ],
            ),
            SubMenu::new(
                " Options ",
                vec![MenuItem {
                    title: "Panel options".into(),
                    highlighted: false,
                }],
            ),
            SubMenu::new(
                " Right ",
                vec![
                    MenuItem {
                        title: "Sort order".into(),
                        highlighted: false,
                    },
                    MenuItem {
                        title: "Filter".into(),
                        highlighted: false,
                    },
                ],
            ),
        ])
    }
}

impl<'block> StatefulWidget for Menu<'block> {
    type State = MenuState;

    fn render(mut self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        buffer.set_style(area, self.style);
        let mut x = area.left();
        let mut remaining_width = area.right().saturating_sub(x);

        for (_idx, submenu) in state.items.iter().enumerate() {
            let is_selected = submenu.selected;
            let has_children = !submenu.items.is_empty();

            let title_style = if submenu.selected {
                self.selected_item_style
            } else {
                self.style
            };

            if is_selected && has_children {
                let group_width = submenu.width as u16;
                render_dropdown(
                    x,
                    area.y + 1,
                    &submenu.items,
                    group_width,
                    buffer,
                    &mut self,
                );
            }

            let span = Span::styled(submenu.label.as_ref(), title_style);
            buffer.set_span(x, area.y, &span, remaining_width);
            x += span.width() as u16;

            remaining_width = remaining_width.saturating_sub(x);
        }
    }
}

fn render_dropdown(
    x: u16,
    y: u16,
    group: &[MenuItem],
    group_width: u16,
    buffer: &mut Buffer,
    menu: &mut Menu,
) {
    let padding = 2;
    let area = Rect::new(x, y, group_width + padding, (group.len() as u16) + padding);
    let dropdown_area = match menu.submenu_block.take() {
        Some(block) => {
            let inner_area = block.inner(area);
            block.render(area, buffer);
            buffer.set_style(inner_area, menu.item_style);
            inner_area
        }
        None => area,
    };
    //Clear.render(dropdown_area, buffer);

    for (idx, item) in group.iter().enumerate() {
        let item_y = dropdown_area.top() + idx as u16;
        let item_style = if item.highlighted {
            menu.selected_item_style
        } else {
            menu.item_style
        };

        let span = Span::styled(item.title.as_ref(), item_style);
        buffer.set_span(dropdown_area.left(), item_y, &span, group_width);
    }
}
