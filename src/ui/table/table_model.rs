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

    pub(crate) fn files(&self) -> &Vec<DirContent> {
        &self.files
    }

    pub(crate) fn list(&mut self) -> Result<(), Error> {
        match list_dir(&self.cwd) {
            Ok(files) => {
                self.files = files;
                return Ok(());
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn pwd(&self) -> &Path {
        self.cwd.as_path()
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
}
