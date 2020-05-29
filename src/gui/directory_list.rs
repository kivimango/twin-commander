use crate::core::list_dir::{list_dir, DirContent};
use orbtk::prelude::*;
use std::path::{Path, PathBuf};

type FileList = Vec<DirContent>;

#[derive(AsAny, Default)]
struct DirectoryListState {
    count: usize,
    cwd: PathBuf,
    list_view: Entity
}

impl State for DirectoryListState {
    fn init(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
        self.cwd = self.cwd();
        self.list_view =  ctx.entity_of_child("directory_view").unwrap();

        match list_dir(Path::new(".")) {
            Ok(result) => {
                self.count = result.len();
               ctx.get_widget(self.list_view).set::<usize>("count", self.count);
                ctx.widget().set::<FileList>("file_list", result);
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

    fn update(&mut self, _: &mut Registry, _context: &mut Context<'_>) {}
}

impl DirectoryListState {
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
}

widget!(DirectoryList<DirectoryListState> {
    file_list: FileList
});

impl Template for DirectoryList {
    fn template(self, id: Entity, bc: &mut BuildContext) -> Self {
        self.name("DirectoryList")
            .child(
                Stack::create()
                    .orientation("vertical")
                    .child(
                        Grid::create()
                            .columns(Columns::create().repeat("*", 6).build())
                            .rows(Rows::create().row("48").build())
                            .child(
                                Button::create()
                                    .text("Name")
                                    .attach(Grid::column(0)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .text("Extension")
                                    .attach(Grid::column(1)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .text("File type")
                                    .attach(Grid::column(2)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .text("Size")
                                    .attach(Grid::column(3)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .text("Last modified")
                                    .attach(Grid::column(4)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .child(
                                Button::create()
                                    .text("Attributes")
                                    .attach(Grid::column(5)).attach(Grid::row(0))
                                    .build(bc)
                            )
                            .build(bc)
                    )
                    .child(
                        ListView::create()
                            .id("directory_view")
                            .element("directory_view")
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
                                            .text(item.name)
                                            .attach(Grid::column(0))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .text(item.ext)
                                            .attach(Grid::column(1))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .text(item.is_dir.to_string())
                                            .attach(Grid::column(2))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .text(item.size)
                                            .attach(Grid::column(3))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .child(
                                        TextBlock::create()
                                            .text(item.date)
                                            .attach(Grid::column(4))
                                            .attach(Grid::row(0))
                                            .build(build_context),
                                    )
                                    .build(build_context)
                            })
                            .count(0)
                            .build(bc)
                        //).build(bc)
                    )
                    .build(bc)
            )
    }
}
