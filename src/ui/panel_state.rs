use crate::core::list_dir::{list_dir, DirContent, FilterOptions};
use crate::core::sort::{TableSortDirection, TableSortPredicate, TableSorter};
use std::io;
use std::path::{Path, PathBuf};

const SORTED_BY_NAME_ASC: usize = 0;
const SORTED_BY_SIZE_ASC: usize = 1;
const SORTED_BY_LASTMODIFIED_ASC: usize = 2;
const SORTED_BY_NAME_DESC: usize = 3;
const SORTED_BY_SIZE_DESC: usize = 4;
const SORTED_BY_LASTMODIFIED_DESC: usize = 5;

const HEADER_LOOKUP_TABLE: [[&str; 3]; 6] = [
    ["Name▼", "Size", "Last modified"],
    ["Name", "Size▼", "Last modified"],
    ["Name", "Size", "Last modified▼"],
    ["Name▲", "Size", "Last modified"],
    ["Name", "Size▲", "Last modified"],
    ["Name", "Size", "Last modified▲"],
];

#[derive(Debug)]
pub enum ChangeDirectoryError {
    AlreadyAtRoot,
    NotADirectory,
    Io(io::Error),
    Other(String),
}

/// The state of the panels in the UI preserved between two view() calls.
pub struct PanelState {
    /// Current working directory
    cwd: PathBuf,

    /// List of files in the `self.cwd` directory
    files: Vec<DirContent>,

    /// Filters used to customize the file listing operation
    filters: FilterOptions,

    /// Sort files and keep track of order and predicate
    sorter: TableSorter,
}

impl PanelState {
    pub fn new() -> Self {
        PanelState::default()
    }

    /// Changes the current working directory based on the selected index.
    ///
    /// This method changes the current working directory based on the index of the selected file or directory.
    /// If the index is 0 and the current directory has a parent directory, it navigates to the parent directory.
    /// If the index corresponds to a directory within the current directory, it navigates to that directory.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the file in the `self.files` collection to navigate to.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the directory change was successful.
    /// - `Err(ChangeDirectoryError)` if an error occurred during the directory change operation.
    ///
    /// # Errors
    ///
    /// This method may return an error in the following cases:
    ///
    /// - `ChangeDirectoryError::Io`: If an I/O error occurs during the directory listing operation.
    /// - `ChangeDirectoryError::AlreadyAtRoot`: If attempting to navigate to the parent directory when already at the root.
    /// - `ChangeDirectoryError::NotADirectory`: If the selected file is not a directory.
    /// - `ChangeDirectoryError::Other`: For other miscellaneous errors during the directory change operation.
    pub fn cd(&mut self, index: usize) -> Result<(), ChangeDirectoryError> {
        if index == 0 {
            if let Some(parent) = self.cwd.parent() {
                match self.list_files(parent) {
                    Ok(mut files) => {
                        self.sorter.sort(&mut files);
                        self.files = files;
                        self.cwd.pop();
                        self.push_parent_front();
                        Ok(())
                    }
                    Err(error) => Err(ChangeDirectoryError::Io(error)),
                }
            } else {
                Err(ChangeDirectoryError::AlreadyAtRoot)
            }
        } else if let Some(selected_file) = self.get_file(index) {
            let file_name = &selected_file.name;
            let mut cwd = self.cwd.clone();
            cwd.push(file_name);

            if cwd.is_dir() {
                match self.list_files(&cwd) {
                    Ok(mut files) => {
                        self.sorter.sort(&mut files);
                        self.files = files;
                        self.cwd = cwd;
                        self.push_parent_front();
                        Ok(())
                    }
                    Err(error) => Err(ChangeDirectoryError::Io(error)),
                }
            } else {
                Err(ChangeDirectoryError::NotADirectory)
            }
        } else {
            Err(ChangeDirectoryError::Other(String::from(
                "no file for index",
            )))
        }
    }

    /// Returns a reference to the list of files found by `self.list_files()`.
    pub fn files(&self) -> &[DirContent] {
        &self.files
    }

    /// Returns a reference to the currently applied filter options.
    pub fn filters(&self) -> &FilterOptions {
        &self.filters
    }

    /// Retrieves the file or directory at the specified index, if it exists.
    pub fn get_file(&self, index: usize) -> Option<&DirContent> {
        self.files.get(index)
    }

    /// Returns the string representation of the current column headers collected into a vector.
    /// The headers also contains the arrow marker indicating the sorting predicate and order.
    pub fn headers(&self) -> Vec<String> {
        header_cells(self.sorter.get_predicate(), self.sorter.get_direction())
    }

    /// List files in a directory at `path` applying the search filters set in `self.filters`.
    pub fn list_files<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DirContent>, io::Error> {
        list_dir(path.as_ref(), &self.filters)
    }

    /// Pushes an entry to the front (index of 0) of the `self.files` vector with the name of ".."
    /// to let the user navigate back to the parent directory of `self.cwd`.
    /// This method should be called after the `self.cwd` contents listed
    /// with `self.list_files()` and after list sorted with `self.sorter.sort()`.
    fn push_parent_front(&mut self) {
        if let Some(_parent) = self.cwd.parent() {
            let parent = DirContent {
                name: String::from(".."),
                size: None,
                is_dir: true,
                date: String::from("<Parent>"),
                attrs: String::new(),
            };
            self.files.insert(0, parent);
        }
    }

    /// Returns the current working directory converted to a String.
    pub fn pwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Sets the current working directory to `path`.
    pub fn set_current_path<P: AsRef<Path>>(&mut self, path: P) {
        self.cwd = PathBuf::from(path.as_ref())
    }

    /// Sets the sorting direction for the files in the panel and sorts them accordingly.
    pub fn set_direction(&mut self, direction: TableSortDirection) {
        self.sorter.set_direction(direction);
        self.sort()
    }

    /// Sets the files to be shown in the ui in the panel and sorts them.
    pub fn set_files(&mut self, files: Vec<DirContent>) {
        self.files = files;
        self.sort();
        self.push_parent_front();
    }

    /// Sets the filtering options during file listing.
    pub fn set_filters(&mut self, filters: FilterOptions) {
        self.filters = filters
    }

    /// Sets the sorting predicate for the files in the panel and sorts them accordingly.
    pub fn set_predicate(&mut self, predicate: TableSortPredicate) {
        self.sorter.set_predicate(predicate);
        self.sort();
    }

    /// Sort `self.files` by the current sorting predicate and direction skipping the first elementh.
    pub fn sort(&mut self) {
        self.sorter.sort(&mut self.files[0..])
    }

    /// Returns the current sorting direction.
    pub fn sort_direction(&self) -> TableSortDirection {
        self.sorter.get_direction()
    }

    /// Returns the current sorting predicate.
    pub fn sort_predicate(&self) -> TableSortPredicate {
        self.sorter.get_predicate()
    }
}

impl Default for PanelState {
    fn default() -> Self {
        PanelState {
            cwd: PathBuf::from("/"),
            files: vec![],
            filters: FilterOptions::default(),
            sorter: TableSorter::new(TableSortDirection::default(), TableSortPredicate::default()),
        }
    }
}

fn header_cells(sorted_by: TableSortPredicate, sort_order: TableSortDirection) -> Vec<String> {
    let header_lookup_index = match sorted_by {
        TableSortPredicate::Name => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_NAME_ASC,
            TableSortDirection::Descending => SORTED_BY_NAME_DESC,
        },
        TableSortPredicate::Size => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_SIZE_ASC,
            TableSortDirection::Descending => SORTED_BY_SIZE_DESC,
        },
        TableSortPredicate::LastModified => match sort_order {
            TableSortDirection::Ascending => SORTED_BY_LASTMODIFIED_ASC,
            TableSortDirection::Descending => SORTED_BY_LASTMODIFIED_DESC,
        },
    };

    HEADER_LOOKUP_TABLE[header_lookup_index]
        .iter()
        .map(|header| String::from(*header))
        .collect::<Vec<String>>()
}
