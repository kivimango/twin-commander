use tui::widgets::TableState;

use crate::core::list_dir::{list_dir, DirContent};
use std::{
    io::Error,
    path::{Path, PathBuf},
};

pub(crate) struct TableViewModel {
    cwd: PathBuf,
    files: Vec<DirContent>,
    state: TableState,
}

impl TableViewModel {
    pub(crate) fn new() -> Self {
        TableViewModel {
            cwd: PathBuf::from("/"),
            files: Vec::new(),
            state: TableState::default(),
        }
    }

    pub(crate) fn cd(&mut self, new_path: PathBuf) -> Result<(), ()> {
        if new_path.is_dir() {
            self.cwd = new_path;
            self.refresh();
            return Ok(());
        } else {
            return Err(());
        }
    }

    pub(crate) fn files(&self) -> &Vec<DirContent> {
        &self.files
    }

    pub(crate) fn list(&mut self) -> Result<(), Error> {
        match list_dir(&self.cwd) {
            Ok(mut files) => {
                // if the dir is not the root, add an .. entry to the result into the first place,
                // to able to go back up one level
                if let Some(_parent) = self.cwd.parent() {
                    let parent = DirContent {
                        name: String::from(".."),
                        ext: String::new(),
                        size: String::from("<DIR>"),
                        is_dir: true,
                        date: String::from("Date"),
                        attrs: String::new(),
                    };
                    files.insert(0, parent);
                }
                self.files = files;
                return Ok(());
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn get_file(&self, index: usize) -> Option<&DirContent> {
        self.files.get(index)
    }

    pub(crate) fn get_file_mut(&mut self, index: usize) -> Option<&mut DirContent> {
        self.files.get_mut(index)
    }

    pub(crate) fn pwd(&self) -> &Path {
        self.cwd.as_path()
    }

    pub(crate) fn set_cwd(&mut self, new_cwd: PathBuf) {
        self.cwd = new_cwd;
    }

    pub(crate) fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub(crate) fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub(crate) fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    pub(crate) fn refresh(&mut self) {
        if let Ok(files) = list_dir(&self.cwd) {
            self.files = files;
        }
    }

    pub(crate) fn reset_selection(&mut self) {
        self.state.select(None);
    }
}
