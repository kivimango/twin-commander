use super::{TableSortDirection, TableSortPredicate, TableSorter};
use crate::core::{
    config::TableConfiguration,
    list_dir::{list_dir, DirContent},
};
use std::{
    io::Error,
    path::{Path, PathBuf},
};
use tui::widgets::TableState;

pub(crate) struct TableViewModel {
    cwd: PathBuf,
    files: Vec<DirContent>,
    state: TableState,
    sorter: TableSorter,
}

impl TableViewModel {
    pub(crate) fn new(table_config: &TableConfiguration) -> Self {
        TableViewModel {
            cwd: table_config.path().clone(),
            files: Vec::new(),
            state: TableState::default(),
            sorter: TableSorter::new(
                TableSortDirection::from(table_config.sort_direction()),
                TableSortPredicate::from(table_config.sort_predicate()),
            ),
        }
    }

    /// Changes `self.cwd` to the currently selected subdirectory or
    /// for the parent of `self.cwd` if it has any.
    /// Returns `Err` when there is no selection,
    /// or the selected item is a file, `Ok` otherwise.
    /// For setting the current working directory for completely different path than `self.cwd`,
    /// use the `set_cwd()` method.
    pub(crate) fn cd(&mut self) -> Result<(), ()> {
        if let Some(selected) = self.selected() {
            if !self.files[selected].is_dir {
                return Err(());
            }

            // the selected item is the parent of the cwd, go back up
            if selected == 0 {
                // the cwd is not the root dir
                if let Some(parent) = self.cwd.parent() {
                    self.set_cwd(parent.to_path_buf());
                    return Ok(());
                }
                // cannot go higher than root
                return Err(());
            }
            // change into the selected dir
            else {
                if let Some(file) = self.get_file(selected) {
                    /*let mut new_path = PathBuf::from(&self.cwd);
                    let dir_name = PathBuf::from(&file.name);
                    new_path.push(dir_name);
                    self.set_cwd(new_path);*/
                    self.cwd.push::<PathBuf>(file.name.clone().into());
                    let _ = self.list();
                    self.select(0);
                    return Ok(());
                }
                return Err(());
            }
        }
        // no selected dir to change
        Err(())
    }

    pub(crate) fn files(&self) -> &Vec<DirContent> {
        &self.files
    }

    pub(crate) fn list(&mut self) -> Result<(), Error> {
        match list_dir(&self.cwd) {
            Ok(files) => {
                self.files = files;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) fn get_file(&self, index: usize) -> Option<&DirContent> {
        self.files.get(index)
    }

    pub(crate) fn pwd(&self) -> &Path {
        self.cwd.as_path()
    }

    /// Pushes an entry to the front (index of 0) of the self.files vector with the name of ".."
    /// to let the user navigate back to the parent directory of self.cwd.
    /// This method should be called after the self.cwd contents listed
    /// with self.list() and after list sorted with sort().
    pub(crate) fn push_parent_front(&mut self) {
        if let Some(_parent) = self.cwd.parent() {
            let parent = DirContent {
                name: String::from(".."),
                size: None,
                is_dir: true,
                date: String::from("Date"),
                attrs: String::new(),
            };
            self.files.insert(0, parent);
        }
    }

    pub(crate) fn set_cwd(&mut self, new_cwd: PathBuf) {
        self.cwd = new_cwd;
    }

    pub(crate) fn select(&mut self, index: usize) {
        if self.files.get(index).is_some() {
            self.state.select(Some(index));
        }
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

    pub(crate) fn sort(&mut self) {
        self.sorter.sort(&mut self.files);
    }

    pub(crate) fn sort_direction(&self) -> TableSortDirection {
        self.sorter.get_direction()
    }

    pub(crate) fn set_sort_direction(&mut self, direction: TableSortDirection) {
        self.sorter.set_direction(direction)
    }

    pub(crate) fn sort_predicate(&self) -> TableSortPredicate {
        self.sorter.get_predicate()
    }

    pub(crate) fn set_sort_predicate(&mut self, predicate: TableSortPredicate) {
        self.sorter.set_predicate(predicate)
    }

    pub(crate) fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    pub(crate) fn refresh(&mut self) {
        if let Ok(files) = list_dir(&self.cwd) {
            self.files = files;
        }
    }

    pub(crate) fn _reset_selection(&mut self) {
        self.state.select(None);
    }
}
