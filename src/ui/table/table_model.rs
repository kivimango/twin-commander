use crate::core::list_dir::{list_dir, DirContent};
use std::{
    io::Error,
    path::{Path, PathBuf},
};

pub(crate) struct TableViewModel {
    cwd: PathBuf,
}

impl TableViewModel {
    pub(crate) fn new() -> Self {
        TableViewModel {
            cwd: PathBuf::from("/"),
        }
    }

    pub(crate) fn list(&self) -> Result<Vec<DirContent>, Error> {
        list_dir(&self.cwd)
    }

    pub(crate) fn pwd(&self) -> &Path {
        self.cwd.as_path()
    }
}
