use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Sometimes this language is a joke: it can't concatenate a string literal and a const str at compile time...

/// The filename of the configuration file.
pub const CONFIG_FILE_NAME: &str = "config.toml";
/// The directory name of the configuration are lives in.
pub const CONFIG_DIR: &str = "twc";
/// The path of the configuration file relative to the user's config directory.
pub const CONFIG_FILE_PATH: &str = "twc/config.toml";
/// The fallback path for the configuration file if the user's config directory not available.
pub const CONFIG_FILE_PATH_FALLBACK: &str = ".";
/// A fallback default path for the `TableView` to list its contents, if the configuration file is missing the `path` key.
pub const TABLE_FALLBACK_PATH: &str = "/";
/// A fallback sort predicate value for the `TableSorter`, if the configuration file is missing the `sort_predicate` key.
pub const TABLE_FALLBACK_PREDICATE: &str = "name";
/// A fallback sort direction value for the `TableSorter`, if the configuration file is missing the `sort_direction` key.
pub const TABLE_FALLBACK_DIRECTION: &str = "asc";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TableConfiguration {
    #[serde(default = "fallback_path")]
    path: PathBuf,
    #[serde(default = "fallback_predicate")]
    sort_predicate: String,
    #[serde(default = "fallback_direction")]
    sort_direction: String,
}

impl TableConfiguration {
    /// Returns the last visited path by the user.
    /// On instantiation, `TableView` will list this folder's contents.
    /// On deserialization errors, this value will be fall back to `TABLE_FALLBACK_PATH`.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    /// Returns the last saved String representation of the `TableSortPredicate`.
    /// `UserInterface` will convert this value into a proper `TableSortPredicate` type on instantiation of a `TableView`.
    pub fn sort_predicate(&self) -> &String {
        &self.sort_predicate
    }

    /// Sets the String representation of the sorting predicate `TableSortPredicate`.
    pub fn set_predicate(&mut self, predicate: String) {
        self.sort_predicate = predicate;
    }

    /// Returns the last saved String representation of the `TableSortDirection`.
    /// `UserInterface` will convert this value into a proper `TableSortdirection` type on instantiation of a `TableView`.
    pub fn sort_direction(&self) -> &String {
        &self.sort_direction
    }

    /// Sets the String representation of the sorting direction `TableSortdirection`.
    pub fn set_sort_direction(&mut self, direction: String) {
        self.sort_direction = direction;
    }
}

impl Default for TableConfiguration {
    /// Returns an instance with sane, fall back values (see the constants section of this module).
    fn default() -> Self {
        TableConfiguration {
            path: TABLE_FALLBACK_PATH.into(),
            sort_predicate: String::from(TABLE_FALLBACK_PREDICATE),
            sort_direction: String::from(TABLE_FALLBACK_DIRECTION),
        }
    }
}

/// A collection of runtime variables that alters the behavior of the application.
#[derive(Serialize, Default, Deserialize)]
pub struct Configuration {
    /// The distinct configuration of the left panel
    left_table: TableConfiguration,
    /// The distinct configuration of the right panel
    right_table: TableConfiguration,
}

impl Configuration {
    /// Returns a reference for the confiugration of the left panel.
    pub fn left_table_config(&self) -> &TableConfiguration {
        &self.left_table
    }

    pub fn left_table_config_mut(&mut self) -> &mut TableConfiguration {
        &mut self.left_table
    }

    /// Returns a reference for the confiugration of the right panel.
    pub fn right_table_config(&self) -> &TableConfiguration {
        &self.right_table
    }

    pub fn right_table_config_mut(&mut self) -> &mut TableConfiguration {
        &mut self.right_table
    }
}

/// Attempts to deserialize a `Configuration` from a configuration file.
///
/// # Errors
///
/// This function will return an error if any of the I/O errors occurs,
/// or the deserialization fails.
pub fn try_load_from_file() -> Result<Configuration, Box<dyn std::error::Error>> {
    let config_path = config_file_path();
    let config_file_contents = std::fs::read_to_string(config_path)?;
    let config = toml::from_str::<Configuration>(&config_file_contents)?;
    Ok(config)
}

/// Attempts to serialize a `Configuration` to a configuration file.
///
/// # Errors
///
/// This function will return an error if any of the I/O errors occurs,
/// or the serialization fails.
pub fn try_save_to_file(config: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = config_file_path();
    let config_serialized = toml::to_string(&config)?;
    std::fs::write(&config_file_path, &config_serialized)?;
    Ok(())
}

/// Returns the path of the configuration file.
/// It first looks on the user's configuration directory (~/.config),
/// if it is not available, it will fall back to the directory of the executable started in, e.g: (.)
pub fn config_file_path() -> PathBuf {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push(CONFIG_FILE_PATH);
        config_dir
    } else {
        let mut current_dir = PathBuf::from(CONFIG_FILE_PATH_FALLBACK);
        current_dir.push(CONFIG_FILE_NAME);
        current_dir
    }
}

/// Returns true if the subdirectory for the configuration file exists.
/// Returns false otherwise, even if the user's configuration directory not available)
pub fn is_dir_exists() -> bool {
    dirs::config_dir().map_or_else(
        || false,
        |mut config_dir| {
            config_dir.push(CONFIG_DIR);
            config_dir.exists()
        },
    )
}

/// Returns true if the the configuration file exists.
/// Returns false otherwise, even if the application's configuration subdirectory,
/// or the user's configuration directory not available)
pub fn is_file_exists() -> bool {
    dirs::config_dir().map_or_else(
        || false,
        |mut config_dir| {
            config_dir.push(CONFIG_FILE_PATH);
            config_dir.exists()
        },
    )
}

/// Attempts to create a subdirectory in the user's configuration directory.
pub fn create_config_dir() -> std::io::Result<()> {
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push(CONFIG_DIR);
        std::fs::create_dir(config_dir)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Users configuration directory not available!",
        ))
    }
}

fn fallback_path() -> PathBuf {
    PathBuf::from(TABLE_FALLBACK_PATH)
}

fn fallback_predicate() -> String {
    String::from(TABLE_FALLBACK_PREDICATE)
}

fn fallback_direction() -> String {
    String::from(TABLE_FALLBACK_DIRECTION)
}

#[cfg(test)]
mod test {
    use super::{
        Configuration, TableConfiguration, TABLE_FALLBACK_DIRECTION, TABLE_FALLBACK_PATH,
        TABLE_FALLBACK_PREDICATE,
    };
    use std::path::PathBuf;

    #[test]
    fn test_table_configuration_default() {
        let table_config = TableConfiguration::default();

        assert_eq!(PathBuf::from(TABLE_FALLBACK_PATH), *table_config.path());
        assert_eq!(TABLE_FALLBACK_PREDICATE, table_config.sort_predicate());
        assert_eq!(TABLE_FALLBACK_DIRECTION, table_config.sort_direction());
    }

    #[test]
    fn test_configuration_default() {
        let config = Configuration::default();
        let table_config = TableConfiguration::default();

        assert_eq!(*config.left_table_config(), table_config);
        assert_eq!(*config.right_table_config(), table_config);
    }
}
