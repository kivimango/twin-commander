use gui::directory_list::DirectoryList;
use orbtk::prelude::*;

mod gui;
mod core;

fn main() {
    Application::new()
        .window(|context| {
            Window::create()
                .title("Twin Commander")
                .position((100.0, 100.0))
                .size(1920.0 * 0.75, 1080.0 * 0.75)
                .resizeable(true)
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
