use crate::core::list_dir::DirContent;
use std::cmp::Ordering;

pub(crate) trait SortBy {
    fn sort(&self, files: &mut Vec<DirContent>);
}

/// Specifies the order of the sorting of the rows in the `TableView`.
/// Default is TableSortDirection::Ascending.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TableSortDirection {
    // Values are arranged from the lowest to the highest.
    /// The lowest value will be placed on the top of the
    /// table. Sorting will continue and place the next increasing
    /// value in the row unit until it reaches the highest value that is
    /// placed on the bottom of the table.
    Ascending,
    /// Values are arranged from the highest to the lowest.
    /// The highest value will be placed on the top of the
    /// table. Sorting will continue and place the next decreasing
    /// value in the row unit until it reaches the lowest value that is
    /// placed on the bottom of the table.
    Descending,
}

impl Default for TableSortDirection {
    fn default() -> Self {
        TableSortDirection::Ascending
    }
}

impl TableSortDirection {
    /// Reversess the current sort order.
    pub fn reverse(&mut self) {
        match self {
            TableSortDirection::Ascending => *self = TableSortDirection::Descending,
            TableSortDirection::Descending => *self = TableSortDirection::Ascending,
        }
    }
}

pub(crate) struct TableSorter {
    direction: TableSortDirection,
    predicate: TableSortPredicate,
    sorter: Box<dyn SortBy>,
}

impl Default for TableSorter {
    fn default() -> Self {
        TableSorter {
            direction: TableSortDirection::default(),
            predicate: TableSortPredicate::default(),
            sorter: Box::new(NameSorterAsc),
        }
    }
}

impl TableSorter {
    pub(crate) fn new() -> Self {
        TableSorter::default()
    }

    pub(crate) fn _with(direction: TableSortDirection, predicate: TableSortPredicate) -> Self {
        TableSorter {
            direction,
            predicate,
            sorter: get_type_by(direction, predicate),
        }
    }

    pub(crate) fn set_direction(&mut self, direction: TableSortDirection) {
        self.direction = direction;
        self.sorter = get_type_by(direction, self.predicate);
    }

    pub(crate) fn set_predicate(&mut self, predicate: TableSortPredicate) {
        self.predicate = predicate;
        self.sorter = get_type_by(self.direction, predicate);
    }

    pub(crate) fn sort(&self, files: &mut Vec<DirContent>) {
        self.sorter.sort(files)
    }
}

fn get_type_by(direction: TableSortDirection, predicate: TableSortPredicate) -> Box<dyn SortBy> {
    match direction {
        TableSortDirection::Ascending => match predicate {
            TableSortPredicate::Name => Box::new(NameSorterAsc),
            TableSortPredicate::Size => Box::new(SizeSorterAsc),
            TableSortPredicate::LastModified => Box::new(LastModifiedSorterAsc),
        },
        TableSortDirection::Descending => match predicate {
            TableSortPredicate::Name => Box::new(NameSorterDesc),
            TableSortPredicate::Size => Box::new(SizeSorterDesc),
            TableSortPredicate::LastModified => Box::new(LastModifiedSorterDesc),
        },
    }
}

/// Defines the column on which the TableView should be sorted by.
/// Default is TableSortPredicate::Name.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TableSortPredicate {
    Name,
    Size,
    LastModified,
}

impl Default for TableSortPredicate {
    fn default() -> Self {
        TableSortPredicate::Name
    }
}

/// It sorts the files in ascending order by name.
/// This sorter is case-sensitive.
pub(crate) struct NameSorterAsc;

impl SortBy for NameSorterAsc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                a.name.cmp(&b.name)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        })
    }
}

/// It sorts the files in descending order by name.
/// This sorter is case-sensitive.
pub(crate) struct NameSorterDesc;

impl SortBy for NameSorterDesc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                b.name.cmp(&a.name)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                b.name.cmp(&a.name)
            }
        })
    }
}

/// It sorts the files in ascending order by size.
pub(crate) struct SizeSorterAsc;

impl SortBy for SizeSorterAsc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                a.size.cmp(&b.size)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                a.size.cmp(&b.size)
            }
        })
    }
}

/// It sorts the files in descending order by size.
pub(crate) struct SizeSorterDesc;

impl SortBy for SizeSorterDesc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                b.size.cmp(&a.size)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                b.size.cmp(&a.size)
            }
        })
    }
}

/// It sorts the files in ascending order by their last modified date.
pub(crate) struct LastModifiedSorterAsc;

impl SortBy for LastModifiedSorterAsc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                a.date.cmp(&b.date)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                a.date.cmp(&b.date)
            }
        })
    }
}

/// It sorts the files in descending order by their last modified date.
pub(crate) struct LastModifiedSorterDesc;

impl SortBy for LastModifiedSorterDesc {
    fn sort(&self, files: &mut Vec<DirContent>) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                b.date.cmp(&a.date)
            } else if a.is_dir && !b.is_dir {
                Ordering::Less
            } else if !a.is_dir && b.is_dir {
                Ordering::Greater
            } else {
                b.date.cmp(&a.date)
            }
        })
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod test {
    use super::{TableSortDirection, TableSortPredicate};
    use crate::core::list_dir::DirContent;

    #[test]
    fn test_sort_direction_default() {
        let sort = TableSortDirection::default();
        assert_eq!(sort, TableSortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_reverse() {
        let mut sort_asc = TableSortDirection::Ascending;
        let mut sort_desc = TableSortDirection::Descending;

        sort_asc.reverse();
        sort_desc.reverse();

        assert_eq!(sort_asc, TableSortDirection::Descending);
        assert_eq!(sort_desc, TableSortDirection::Ascending)
    }

    #[test]
    fn test_sort_by_name_asc() {
        let mut files = setup();

        sort(
            TableSortDirection::Ascending,
            TableSortPredicate::Name,
            &mut files,
        );

        assert_eq!(files[0].name, String::from("Alpha"));
        assert_eq!(files[files.len() - 1].name, String::from("test.txt"));
    }

    #[test]
    fn test_sort_by_name_desc() {
        let mut files = setup();

        sort(
            TableSortDirection::Descending,
            TableSortPredicate::Name,
            &mut files,
        );

        assert_eq!(files[0].name, String::from("Omega"));
        assert_eq!(files[files.len() - 1].name, String::from("a.out"));
    }

    #[test]
    fn test_sort_by_size_asc() {}

    #[test]
    fn test_sort_by_size_desc() {}

    #[test]
    fn test_sort_by_last_modified_asc() {
        let mut files = setup();

        sort(
            TableSortDirection::Ascending,
            TableSortPredicate::LastModified,
            &mut files,
        );

        assert_eq!(files[0].date, String::from("2022.11.23 11:03:01"));
        assert_eq!(
            files[files.len() - 1].date,
            String::from("2022.11.27 15:07:05")
        );
    }

    #[test]
    fn test_sort_by_last_modified_desc() {
        let mut files = setup();

        sort(
            TableSortDirection::Descending,
            TableSortPredicate::LastModified,
            &mut files,
        );

        assert_eq!(files[0].date, String::from("2022.11.27 15:07:05"));
        assert_eq!(
            files[files.len() - 1].date,
            String::from("2022.11.23 11:03:01")
        );
    }

    fn setup() -> Vec<DirContent> {
        let mut files = Vec::new();
        files.push(DirContent {
            name: String::from("Beta"),
            is_dir: true,
            size: String::from("<DIR>"),
            date: String::from("2022.11.24 12:04:02"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("Omega"),
            is_dir: true,
            size: String::from("<DIR>"),
            date: String::from("2022.11.25 13:05:03"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("Alpha"),
            is_dir: true,
            size: String::from("<DIR>"),
            date: String::from("2022.11.23 11:03:01"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("test.txt"),
            is_dir: false,
            size: String::from("816"),
            date: String::from("2022.11.26 14:06:04"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("a.out"),
            is_dir: false,
            size: String::from("8467"),
            date: String::from("2022.11.27 15:07:05"),
            attrs: String::new(),
        });
        return files;
    }
}
