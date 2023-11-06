use crate::core::list_dir::DirContent;
use std::cmp::Ordering;

const PREDICATE_NAME: usize = 0;
const PREDICATE_SIZE: usize = 1;
const PREDICATE_LAST_MODIFIED: usize = 2;
const DIRECTION_ASC: usize = 0;
const DIRECTION_DESC: usize = 1;

pub(crate) trait SortBy {
    fn sort(&self, files: &mut [DirContent]);
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

impl From<&String> for TableSortDirection {
    fn from(value: &String) -> Self {
        let value = value.to_lowercase();
        let value = value.as_str();
        if value == "asc" {
            TableSortDirection::Ascending
        } else if value == "desc" {
            TableSortDirection::Descending
        } else {
            TableSortDirection::default()
        }
    }
}

impl From<usize> for TableSortDirection {
    fn from(value: usize) -> Self {
        match value {
            DIRECTION_ASC => TableSortDirection::Ascending,
            DIRECTION_DESC => TableSortDirection::Descending,
            _ => TableSortDirection::default(),
        }
    }
}

impl From<TableSortDirection> for String {
    fn from(value: TableSortDirection) -> Self {
        match value {
            TableSortDirection::Ascending => String::from("asc"),
            TableSortDirection::Descending => String::from("desc"),
        }
    }
}

impl TableSortDirection {
    /// Reverses the current sort order.
    pub fn _reverse(&mut self) {
        match self {
            TableSortDirection::Ascending => *self = TableSortDirection::Descending,
            TableSortDirection::Descending => *self = TableSortDirection::Ascending,
        }
    }

    /// Returns the current variant as an usize value.
    pub fn as_usize(&self) -> usize {
        match self {
            TableSortDirection::Ascending => 0,
            TableSortDirection::Descending => 1,
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
    pub(crate) fn new(direction: TableSortDirection, predicate: TableSortPredicate) -> Self {
        TableSorter {
            direction,
            predicate,
            sorter: get_type_by(direction, predicate),
        }
    }

    pub(crate) fn get_direction(&self) -> TableSortDirection {
        self.direction
    }

    pub(crate) fn get_predicate(&self) -> TableSortPredicate {
        self.predicate
    }

    pub(crate) fn set_direction(&mut self, direction: TableSortDirection) {
        self.direction = direction;
        self.sorter = get_type_by(direction, self.predicate);
    }

    pub(crate) fn set_predicate(&mut self, predicate: TableSortPredicate) {
        self.predicate = predicate;
        self.sorter = get_type_by(self.direction, predicate);
    }

    pub(crate) fn sort(&self, files: &mut [DirContent]) {
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

impl From<&String> for TableSortPredicate {
    fn from(value: &String) -> Self {
        let value = value.to_lowercase();
        if value == "name" {
            TableSortPredicate::Name
        } else if value == "size" {
            TableSortPredicate::Size
        } else if value == "modified" {
            TableSortPredicate::LastModified
        } else {
            TableSortPredicate::default()
        }
    }
}

impl From<TableSortPredicate> for String {
    fn from(value: TableSortPredicate) -> Self {
        match value {
            TableSortPredicate::Name => String::from("name"),
            TableSortPredicate::Size => String::from("size"),
            TableSortPredicate::LastModified => String::from("modified"),
        }
    }
}

impl From<usize> for TableSortPredicate {
    fn from(value: usize) -> Self {
        match value {
            PREDICATE_NAME => TableSortPredicate::Name,
            PREDICATE_SIZE => TableSortPredicate::Size,
            PREDICATE_LAST_MODIFIED => TableSortPredicate::LastModified,
            _ => TableSortPredicate::default(),
        }
    }
}

impl TableSortPredicate {
    pub fn as_usize(&self) -> usize {
        match self {
            TableSortPredicate::Name => PREDICATE_NAME,
            TableSortPredicate::Size => PREDICATE_SIZE,
            TableSortPredicate::LastModified => PREDICATE_LAST_MODIFIED,
        }
    }
}

/// It sorts the files in ascending order by name.
/// This sorter is case-sensitive.
pub(crate) struct NameSorterAsc;

impl SortBy for NameSorterAsc {
    fn sort(&self, files: &mut [DirContent]) {
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
    fn sort(&self, files: &mut [DirContent]) {
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
    fn sort(&self, files: &mut [DirContent]) {
        files.sort_by(|a, b| {
            if a.is_dir && b.is_dir {
                b.size.cmp(&a.size)
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
    fn sort(&self, files: &mut [DirContent]) {
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
    fn sort(&self, files: &mut [DirContent]) {
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
    fn sort(&self, files: &mut [DirContent]) {
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
    use super::*;
    use crate::core::list_dir::DirContent;

    #[test]
    fn test_sort_direction_default() {
        let sort = TableSortDirection::default();
        assert_eq!(sort, TableSortDirection::Ascending);
    }

    #[test]
    fn test_predicate_default() {
        let predicate = TableSortPredicate::default();
        assert_eq!(predicate, TableSortPredicate::Name);
    }

    #[test]
    fn test_sorter_default() {
        let sorter = TableSorter::default();
        assert_eq!(sorter.get_direction(), TableSortDirection::Ascending);
        assert_eq!(sorter.get_predicate(), TableSortPredicate::Name);
    }

    #[test]
    fn test_sort_direction_reverse() {
        let mut sort_asc = TableSortDirection::Ascending;
        let mut sort_desc = TableSortDirection::Descending;

        sort_asc._reverse();
        sort_desc._reverse();

        assert_eq!(sort_asc, TableSortDirection::Descending);
        assert_eq!(sort_desc, TableSortDirection::Ascending)
    }

    #[test]
    fn test_sort_by_name_asc() {
        let mut files = setup();
        let sorter = NameSorterAsc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].name, String::from("Alpha"));
        assert_eq!(files[2].name, String::from("Omega"));

        //then files
        assert_eq!(files[3].name, String::from("a.out"));
        assert_eq!(files[files.len() - 1].name, String::from("test.txt"));
    }

    #[test]
    fn test_sort_by_name_desc() {
        let mut files = setup();
        let sorter = NameSorterDesc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].name, String::from("Omega"));
        assert_eq!(files[2].name, String::from("Alpha"));

        //then files
        assert_eq!(files[3].name, String::from("test.txt"));
        assert_eq!(files[files.len() - 1].name, String::from("a.out"));
    }

    #[test]
    fn test_sort_by_size_asc() {
        let mut files = setup();
        let sorter = SizeSorterAsc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].size, None);
        assert_eq!(files[1].size, None);
        assert_eq!(files[2].size, None);

        // then files
        assert_eq!(files[3].size, Some(816));
        assert_eq!(files[4].size, Some(8467));
    }

    #[test]
    fn test_sort_by_size_desc() {
        let mut files = setup();
        let sorter = SizeSorterDesc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].size, None);
        assert_eq!(files[1].size, None);
        assert_eq!(files[2].size, None);

        // then files
        assert_eq!(files[3].size, Some(8467));
        assert_eq!(files[4].size, Some(816));
    }

    #[test]
    fn test_sort_by_last_modified_asc() {
        let mut files = setup();
        let sorter = LastModifiedSorterAsc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].date, String::from("2022.11.23 11:03:01"));
        assert_eq!(files[2].date, String::from("2022.11.25 13:05:03"));

        // then files
        assert_eq!(files[3].date, String::from("2022.11.26 14:06:04"));
        assert_eq!(
            files[files.len() - 1].date,
            String::from("2022.11.27 15:07:05")
        );
    }

    #[test]
    fn test_sort_by_last_modified_desc() {
        let mut files = setup();
        let sorter = LastModifiedSorterDesc;

        sorter.sort(&mut files);

        // directories first
        assert_eq!(files[0].date, String::from("2022.11.25 13:05:03"));
        assert_eq!(files[2].date, String::from("2022.11.23 11:03:01"));

        // then files
        assert_eq!(files[3].date, String::from("2022.11.27 15:07:05"));
        assert_eq!(
            files[files.len() - 1].date,
            String::from("2022.11.26 14:06:04")
        );
    }

    fn setup() -> Vec<DirContent> {
        let mut files = Vec::new();
        files.push(DirContent {
            name: String::from("Beta"),
            is_dir: true,
            size: None,
            date: String::from("2022.11.24 12:04:02"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("Omega"),
            is_dir: true,
            size: None,
            date: String::from("2022.11.25 13:05:03"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("Alpha"),
            is_dir: true,
            size: None,
            date: String::from("2022.11.23 11:03:01"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("test.txt"),
            is_dir: false,
            size: Some(816),
            date: String::from("2022.11.26 14:06:04"),
            attrs: String::new(),
        });
        files.push(DirContent {
            name: String::from("a.out"),
            is_dir: false,
            size: Some(8467),
            date: String::from("2022.11.27 15:07:05"),
            attrs: String::new(),
        });
        return files;
    }

    #[test]
    fn test_from_string_on_sort_direction_asc() {
        let from_uppercase = String::from("ASC");
        let direction = TableSortDirection::from(&from_uppercase);
        assert_eq!(direction, TableSortDirection::Ascending);

        let from_lowercase = String::from("asc");
        let direction = TableSortDirection::from(&from_lowercase);
        assert_eq!(direction, TableSortDirection::Ascending);

        let mixed_case = String::from("AsC");
        let direction = TableSortDirection::from(&mixed_case);
        assert_eq!(direction, TableSortDirection::Ascending);
    }

    #[test]
    fn test_from_string_on_sort_direction_desc() {
        let from_uppercase = String::from("DESC");
        let direction = TableSortDirection::from(&from_uppercase);
        assert_eq!(direction, TableSortDirection::Descending);

        let from_lowercase = String::from("desc");
        let direction = TableSortDirection::from(&from_lowercase);
        assert_eq!(direction, TableSortDirection::Descending);

        let mixed_case = String::from("dEsC");
        let direction = TableSortDirection::from(&mixed_case);
        assert_eq!(direction, TableSortDirection::Descending);
    }

    #[test]
    fn test_from_string_on_sort_direction_default() {
        let invalid_value = String::from("notavalidvalue");
        let direction = TableSortDirection::from(&invalid_value);
        assert_eq!(direction, TableSortDirection::default());
    }

    #[test]
    fn test_from_string_on_sort_predicate_name() {
        let from_uppercase = String::from("NAME");
        let predicate = TableSortPredicate::from(&from_uppercase);
        assert_eq!(predicate, TableSortPredicate::Name);

        let from_lowercase = String::from("name");
        let predicate = TableSortPredicate::from(&from_lowercase);
        assert_eq!(predicate, TableSortPredicate::Name);

        let mixed_case = String::from("NaMe");
        let predicate = TableSortPredicate::from(&mixed_case);
        assert_eq!(predicate, TableSortPredicate::Name);
    }

    #[test]
    fn test_from_string_on_sort_predicate_size() {
        let from_uppercase = String::from("SIZE");
        let predicate = TableSortPredicate::from(&from_uppercase);
        assert_eq!(predicate, TableSortPredicate::Size);

        let from_lowercase = String::from("size");
        let predicate = TableSortPredicate::from(&from_lowercase);
        assert_eq!(predicate, TableSortPredicate::Size);

        let mixed_case = String::from("SiZe");
        let predicate = TableSortPredicate::from(&mixed_case);
        assert_eq!(predicate, TableSortPredicate::Size);
    }

    #[test]
    fn test_from_string_on_sort_predicate_last_modified() {
        let from_uppercase = String::from("MODIFIED");
        let predicate = TableSortPredicate::from(&from_uppercase);
        assert_eq!(predicate, TableSortPredicate::LastModified);

        let from_lowercase = String::from("modified");
        let predicate = TableSortPredicate::from(&from_lowercase);
        assert_eq!(predicate, TableSortPredicate::LastModified);

        let mixed_case = String::from("MoDiFiEd");
        let predicate = TableSortPredicate::from(&mixed_case);
        assert_eq!(predicate, TableSortPredicate::LastModified);
    }

    #[test]
    fn test_from_string_on_sort_predicate_default() {
        let invalid_input = String::from("invalidinput");
        let predicate = TableSortPredicate::from(&invalid_input);
        assert_eq!(predicate, TableSortPredicate::default());
    }

    #[test]
    fn test_into_string_on_sort_direction() {
        let direction_asc = TableSortDirection::Ascending;
        let direction_desc = TableSortDirection::Descending;
        let into_asc: String = direction_asc.into();
        let into_desc: String = direction_desc.into();

        assert_eq!(into_asc, String::from("asc"));
        assert_eq!(into_desc, String::from("desc"));
    }

    #[test]
    fn test_into_string_on_sort_predicate() {
        let name: String = TableSortPredicate::Name.into();
        let size: String = TableSortPredicate::Size.into();
        let last_modified: String = TableSortPredicate::LastModified.into();

        assert_eq!(name, String::from("name"));
        assert_eq!(size, String::from("size"));
        assert_eq!(last_modified, String::from("modified"));
    }

    #[test]
    fn test_predicate_to_usize() {
        assert_eq!(TableSortPredicate::Name.as_usize(), PREDICATE_NAME);
        assert_eq!(TableSortPredicate::Size.as_usize(), PREDICATE_SIZE);
        assert_eq!(
            TableSortPredicate::LastModified.as_usize(),
            PREDICATE_LAST_MODIFIED
        );
    }

    #[test]
    fn test_direction_to_usize() {
        assert_eq!(TableSortDirection::Ascending.as_usize(), DIRECTION_ASC);
        assert_eq!(TableSortDirection::Descending.as_usize(), DIRECTION_DESC);
    }

    #[test]
    fn test_predicate_from_usize() {
        assert_eq!(
            TableSortPredicate::from(PREDICATE_NAME),
            TableSortPredicate::Name
        );
        assert_eq!(
            TableSortPredicate::from(PREDICATE_SIZE),
            TableSortPredicate::Size
        );
        assert_eq!(
            TableSortPredicate::from(PREDICATE_LAST_MODIFIED),
            TableSortPredicate::LastModified
        );
    }

    #[test]
    fn test_direction_from_usize() {
        assert_eq!(
            TableSortDirection::from(DIRECTION_ASC),
            TableSortDirection::Ascending
        );
        assert_eq!(
            TableSortDirection::from(DIRECTION_DESC),
            TableSortDirection::Descending
        );
    }
}
