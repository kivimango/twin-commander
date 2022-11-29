use chrono::{DateTime, Local};
use std::fs;
use std::io::Error;
use std::path::Path;

/// A structure representing one file with its metadata collected from listing files in a directory
#[derive(Clone, Debug, PartialEq)]
pub struct DirContent {
    pub name: String,
    pub is_dir: bool,
    pub size: String,
    pub date: String,
    pub attrs: String,
}

/// Collects list of files and directories with their metadata in the given `path`
/// into a vector of `DirContent`.
/// I/O errors related to the `path` are propagated up to the caller.
/// I/O errors related to filename and last modification date will not interrupt listing,
/// instead those values will be replaced with a placeholder indicating that the function failed to
/// read those values.
pub fn list_dir(dir: &Path) -> Result<Vec<DirContent>, Error> {
    let mut result: Vec<DirContent> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let dir = entry?;
        let metadata = dir.metadata()?;

        let (name, is_dir, size, date, attrs) = {
            let f_name = match dir.file_name().into_string() {
                Ok(name) => name,
                Err(e) => {
                    eprintln!("NOTICE: cannot read filename, error: {:?}", e);
                    "N/A".to_string()
                }
            };

            let mut is_dir = true;
            let mut size = String::from("<DIR>");

            if !metadata.is_dir() {
                is_dir = false;
                size = metadata.len().to_string();
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
                        &f_name, e
                    );
                    "N/A".to_string()
                }
            };

            let attrs = "".to_string();

            (f_name, is_dir, size, date, attrs)
        };

        let file_details = DirContent {
            name,
            is_dir,
            size,
            date,
            attrs,
        };
        result.push(file_details);
    }

    return Ok(result);
}
