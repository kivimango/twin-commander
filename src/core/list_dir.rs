use chrono::{DateTime, Local};
use std::fs::{self, DirEntry};
use std::io::Error;
use std::path::Path;

/// A structure representing one file with its metadata collected from listing files in a directory
#[derive(Clone, Debug, PartialEq)]
pub struct DirContent {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub date: String,
    pub attrs: String,
}

impl From<DirEntry> for DirContent {
    fn from(entry: DirEntry) -> Self {
        let name = match entry.file_name().into_string() {
            Ok(fname) => fname,
            Err(error) => {
                eprintln!("NOTICE: cannot read filename: {:?}", error);
                "N/A".to_string()
            }
        };

        let mut is_dir = true;
        let mut size = None;
        let (is_dir, size, date, attrs) = match entry.metadata() {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    is_dir = false;
                    size = Some(metadata.len());
                }

                let date = match metadata.modified() {
                    Ok(modified) => {
                        let datetime_local: DateTime<Local> = modified.into();
                        let dt_formatted = datetime_local.format("%Y.%m.%d %H:%M");
                        dt_formatted.to_string()
                    }
                    Err(e) => {
                        eprintln!(
                            "NOTICE: cannot read last modification date for {}, error: {}",
                            &name, e
                        );
                        "N/A".to_string()
                    }
                };

                let attrs = "".to_string();

                (is_dir, size, date, attrs)
            }
            Err(error) => {
                eprintln!("NOTICE: cannot read file metadata: {:?}", error);
                (is_dir, None, String::new(), String::new())
            }
        };

        DirContent {
            name,
            is_dir,
            size,
            date,
            attrs,
        }
    }
}

#[derive(Default)]
pub struct FilterOptions {
    pub show_hidden_files: bool,
}

#[allow(unused)]
impl FilterOptions {
    pub fn new() -> Self {
        FilterOptions::default()
    }

    pub fn show_hidden_files(&self) -> bool {
        self.show_hidden_files
    }

    pub fn set_show_hidden_files(&mut self, show: bool) {
        self.show_hidden_files = show;
    }
}

/// Collects list of files and directories with their metadata in the given `path`
/// into a vector of `DirContent`.
/// I/O errors related to the `path` are propagated up to the caller.
/// I/O errors related to filename and last modification date will not interrupt listing,
/// instead those values will be replaced with a placeholder indicating that the function failed to
/// read those values.
pub fn list_dir(dir: &Path, filter_options: &FilterOptions) -> Result<Vec<DirContent>, Error> {
    let result: Vec<DirContent> = fs::read_dir(dir)?
        .filter_map(|result| result.ok())
        .filter(|entry| {
            filter_options.show_hidden_files
                || !entry.file_name().to_string_lossy().starts_with('.')
        })
        .map(DirContent::from)
        .collect();

    Ok(result)
}
