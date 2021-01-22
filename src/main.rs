use gui::directory_list::DirectoryList;
use orbtk::{
    prelude::*,
    theme_default::{THEME_DEFAULT, THEME_DEFAULT_COLORS_DARK, THEME_DEFAULT_FONTS},
    theming::config::ThemeConfig,
};

mod gui;
mod core;

static DEFAULT_DARK_EXT: &str = include_str!("../assets/twin-commander/twin_commander_default_dark.ron");

fn theme() -> Theme {
    register_default_fonts(Theme::from_config(
        ThemeConfig::from(DEFAULT_DARK_EXT)
            .extend(ThemeConfig::from(THEME_DEFAULT))
            .extend(ThemeConfig::from(THEME_DEFAULT_COLORS_DARK))
            .extend(ThemeConfig::from(THEME_DEFAULT_FONTS)),
    ))
}

fn main() {
    Application::new()
        .theme(theme())
        .window(|context| {
            Window::new()
                .title("Twin Commander")
                .position((100.0, 100.0))
                .size(1920.0 * 0.775, 1080.0 * 0.75)
                .resizeable(true)
                .child(
                    Stack::new()
                        .orientation("horizontal")
                        .spacing(1.0)
                        .child(DirectoryList::new().build(context))
                        .child(DirectoryList::new().build(context))
                        .build(context),
                )
                .build(context)
        })
        .run();
}
