use gui::directory_list::DirectoryList;
use orbtk::prelude::*;

mod gui;
mod core;

static DEFAULT_THEME: &'static str = include_str!("../res/theme/default.css");

fn get_theme() -> ThemeValue {
    ThemeValue::create()
        .extension_css(theme::DEFAULT_THEME_CSS)
        .extension_css(DEFAULT_THEME)
        .build()
}

fn main() {
    Application::new()
        .window(|context| {
            Window::create()
                .title("Twin Commander")
                .position((100.0, 100.0))
                .size(1920.0 * 0.775, 1080.0 * 0.75)
                .resizeable(true)
                .theme(get_theme())
                .child(
                    Stack::create()
                        .orientation("horizontal")
                        .spacing(1.0)
                        .child(DirectoryList::create().build(context))
                        .child(DirectoryList::create().build(context))
                        .build(context),
                )
                .build(context)
        })
        .run();
}
