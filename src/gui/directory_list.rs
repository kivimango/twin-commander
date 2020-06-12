use crate::core::list_dir::{list_dir, DirContent};
use orbtk::prelude::*;
use orbtk::behaviors::MouseBehavior;
use orbtk::shell::{Key, KeyEvent};
use std::path::PathBuf;

type FileList = Vec<DirContent>;

//const DIRECTORY_LIST_ID: &'static str = "directory_list";
const CWD_LABEL_ID: &'static str = "path_label";

#[derive(Clone)]
enum DirectoryListAction {
    Key(KeyEvent),
    RequestFocus
}

#[derive(AsAny, Default)]
struct DirectoryListState {
    action: Option<DirectoryListAction>,
    count: usize,
    cwd: PathBuf,
    list_view: Entity,
    path_label: Entity,
    selected_item_index: usize
}

impl State for DirectoryListState {
    fn init(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
        self.cwd = self.cwd();
        // TODO: fix ListView custom-id-breaks-selection issue
        self.list_view =  ctx.entity_of_child("list_view").unwrap();
        self.path_label = ctx.entity_of_child(CWD_LABEL_ID).unwrap();

        match list_dir(self.cwd().as_path()) {
            Ok(result) => {
                self.count = result.len();
                ctx.get_widget(self.list_view).set::<usize>("count", self.count);
                ctx.widget().set::<FileList>("file_list", result);
                ctx.get_widget(self.path_label).set::<String16>("text", String16::from(self.cwd.to_str().unwrap()));
            }
            // TODO: show popup
            Err(error) => {
                eprintln!(
                    "NOTICE: Error during listing of files in {:#?}, : {}",
                    self.cwd, error
                );
            }
        }
    }

    fn update(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
        if let Some(action) = self.action.clone() {
            match action {
                DirectoryListAction::Key(key_event) => {
                    match key_event.key {
                        Key::Up => {
                            // move the selection only if there is one list item selected
                            if self.selected_item_count(ctx) == 1 {
                                self.handle_up_key(ctx);
                            }
                        },
                        Key::Down => {
                            // move the selection only if there is one list item selected
                            if self.selected_item_count(ctx) == 1 {
                                self.handle_down_key(ctx);
                            }
                        },
                        _ => {}
                    }
                },
                DirectoryListAction::RequestFocus => {
                    self.request_focus(ctx);
                }
            }
            self.action = None;
        }
    }
}

impl DirectoryListState {
    fn action(&mut self, action: DirectoryListAction) {
        self.action = Some(action);
    }

    fn cwd(&self) -> PathBuf {
        return match std::env::current_dir() {
            // TODO: save last visited dir, continue from there (load on start)
            Ok(content) => content,
            // TODO: show popup
            // fallback to root
            Err(e) => {
                eprintln!("NOTICE: error during reading {:#?} : {}", self.cwd, e);
                PathBuf::from("/")
            }
        };
    }

    fn handle_up_key(&mut self, ctx: &mut Context<'_>) {
        if self.selected_item_index > 0 && self.selected_item_index <= self.count {
            self.move_selection(self.selected_item_index - 1, ctx);
        }
    }

    fn handle_down_key(&mut self, ctx: &mut Context<'_>) {
        if self.selected_item_index <= self.count && (self.count - 1) != self.selected_item_index {
            self.move_selection(self.selected_item_index + 1, ctx);
        }
    }

    fn move_selection(&mut self, new_index: usize, ctx: &mut Context<'_>) {
        match ctx.entity_of_child("items_panel") {
            Some(list_items_panel) => {
                // changing the current context into ListView's items_panel
                ctx.entity = list_items_panel;
                self.deselect_current_item(ctx);
                self.select_item(new_index, ctx);
            },
            None => {
                eprintln!("NOTICE: could not get list view items panel");
            }
        }
    }

    fn select_item(&mut self, new_index: usize, ctx: &mut Context<'_>) {
        self.selected_item_index = new_index;
        let mut should_add = false;
        let mut child_entity = Entity::default();

        if let Some(mut child) = ctx.try_child_from_index(self.selected_item_index) {
            // probably a bug in orbtk's ListViewItemState's update_post_layout, should be set to true
            child.set("selected", false);
            should_add = true;
            child_entity = child.entity();
        }

        if should_add {
            ctx.get_widget(self.list_view)
                .get_mut::<SelectedIndices>("selected_indices")
                .0.insert(self.selected_item_index);
            ctx.get_widget(self.list_view)
                .get_mut::<SelectedEntities>("selected_entities")
                .0.insert(child_entity);
        }
    }

    fn deselect_current_item(&self, ctx: &mut Context<'_> ) {
        let mut should_remove = false;
        let mut child_entity= Entity::default() ;

        if let Some(mut child) = ctx.try_child_from_index(self.selected_item_index) {
            // probably a bug in orbtk's ListViewItemState's update_post_layout, should be set to false
            child.set("selected", true);
            child_entity = child.entity();
            should_remove = true;
        }

        if should_remove {
            ctx.get_widget(self.list_view)
                .get_mut::<SelectedIndices>("selected_indices")
                .0.remove(&self.selected_item_index);
            ctx.get_widget(self.list_view)
                .get_mut::<SelectedEntities>("selected_entities")
                .0.remove(&child_entity);
        }
    }

    fn selected_item_count(&self, ctx: &mut Context<'_>) -> usize {
       ctx.get_widget(self.list_view)
           .get::<SelectedEntities>("selected_entities").0.len()
    }

    fn request_focus(&self, ctx: &mut Context<'_>) {
        if !ctx.widget().get::<bool>("focused") {
            ctx.widget().set::<bool>("focused", true);
            ctx.push_event_by_window(FocusEvent::RequestFocus(ctx.entity));
        }

        // TODO: set the selected item index when mouse is clicked on the list
    }
}

widget!(DirectoryList<DirectoryListState>: MouseHandler, KeyDownHandler {
    delta: Point,
    file_list: FileList,
    focused: bool,
    pressed: bool
});

impl Template for DirectoryList {
    fn template(self, id: Entity, bc: &mut BuildContext) -> Self {
        self.name("DirectoryList")
            .child(
                Stack::create()
                    .orientation("vertical")
                    .child(
                        Container::create()
                            .class("cwd_label_container")
                            .child(
                                TextBlock::create()
                                    .class("cwd_label")
                                    .id(CWD_LABEL_ID)
                                    .build(bc)
                            )
                            .build(bc)
                    )
                    .child(
                        Grid::create()
                            .columns(Columns::create().repeat("*", 6).build())
                            .rows(Rows::create().row("48").build())
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("Name")
                                    .attach(Grid::column(0)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("Extension")
                                    .attach(Grid::column(1)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("File type")
                                    .attach(Grid::column(2)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("Size")
                                    .attach(Grid::column(3)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("Last modified")
                                    .attach(Grid::column(4)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .class("directory_view_column_header")
                                    .text("Attributes")
                                    .attach(Grid::column(5)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .build(bc)
                    )
                    .child(
                        ListView::create()
                            //.id(DIRECTORY_LIST_ID)
                            .id("list_view")
                            .class("directory_list")
                            .width(750.0)
                            .height(700.0)
                            .items_builder(move |build_context, index| {
                                let ll = build_context.get_widget(id);
                                let item = ll.get::<FileList>("file_list")[index].clone();

                                Grid::create()
                                    .columns(Columns::create().repeat("*", 6).build())
                                    .rows(Rows::create().row("48").build())
                                    .child(
                                        TextBlock::create()
                                            .element("list-view-item")
                                            .text(item.name)
                                            .attach(Grid::column(0))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .element("list-view-item")
                                            .text(item.ext)
                                            .attach(Grid::column(1))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .element("list-view-item")
                                            .text(item.is_dir.to_string())
                                            .attach(Grid::column(2))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .element("list-view-item")
                                            .text(item.size)
                                            .attach(Grid::column(3))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .element("list-view-item")
                                            .text(item.date)
                                            .attach(Grid::column(4))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .build(build_context)
                            })
                            .count(0)
                            .build(bc)
                    )
                    .build(bc)
            )
            .on_key_down(move |states, key_event| -> bool {
                states.get_mut::<DirectoryListState>(id).action(DirectoryListAction::Key(key_event));
                false
            })
            .child(
                MouseBehavior::create()
                    .on_mouse_down(move |states, _| -> bool {
                        states.get_mut::<DirectoryListState>(id).action(DirectoryListAction::RequestFocus);
                        true
                    })
                    .build(bc)
            )
    }
}
