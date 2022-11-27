use std::fs;
use std::io::Error;
use std::path::Path;

/// A structure representing one file with its metadata collected from listing files in a directory
#[derive(Debug, Clone)]
pub struct DirContent {
    pub name: String,
    pub ext: String,
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

    // if path is not a root, add the parent of the path as a first item to allow the user navigate
    // up in th tree
    /*match dir.parent() {
        Some(parent) => {
            let parent = DirContent {
                name: "..".to_string(),
                ext: String::new(),
                is_dir: true,
                size: String::new(),
                date: String::new(),
                attrs: String::new(),
            };
            result.push(parent);
        }
        None => {}
    }*/

    for entry in fs::read_dir(dir)? {
        let dir = entry?;
        let metadata = dir.metadata()?;

        let (name, ext, is_dir, size, date, attrs) = {
            let f_name = match dir.file_name().into_string() {
                Ok(name) => name,
                Err(e) => {
                    eprintln!("NOTICE: cannot read filename, error: {:?}", e);
                    "N/A".to_string()
                }
            };

            let mut is_dir = true;
            let mut ext = String::new();
            let mut size = String::from("<DIR>");

            if !metadata.is_dir() {
                is_dir = false;
                ext = String::from(extension(f_name.as_str()));
                size = metadata.len().to_string();
            }

            let date = match metadata.modified() {
                // TODO: modified.to_string()
                Ok(_modified) => "Date".to_string(),
                Err(e) => {
                    eprintln!(
                        "NOTICE: cannot read last modification date for {}, error: {}",
                        &f_name, e
                    );
                    "N/A".to_string()
                }
            };

            let attrs = "".to_string();

            (f_name, ext, is_dir, size, date, attrs)
        };

        let file_details = DirContent {
            name,
            ext,
            is_dir,
            size,
            date,
            attrs,
        };
        result.push(file_details);
    }

    return Ok(result);
}

/// Returns the extension of a file or "N/A" if the filename does not contain a period.
pub fn extension(filename: &str) -> &str {
    filename
        .rfind('.')
        .map(|idx| &filename[idx..])
        .filter(|ext| ext.chars().skip(1).all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("N/A")
}
