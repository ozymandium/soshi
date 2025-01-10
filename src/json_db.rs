use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Struct for the json database file
#[derive(Serialize, Deserialize)]
pub struct JsonDb {
    pub conflicts: Vec<PathBuf>,
}

impl JsonDb {
    /// Create a new JsonDb with the specified list of conflicts
    ///
    /// # Arguments
    /// * `conflicts`: list of conflict files
    ///
    /// # Returns
    /// A new JsonDb with the specified list of conflicts
    pub fn new(conflicts: Vec<PathBuf>) -> JsonDb {
        JsonDb { conflicts }
    }

    pub fn load(path: &PathBuf) -> Result<JsonDb> {
        if !path.exists() {
            return Ok(JsonDb::new(Vec::new()));
        }
        let content = fs::read_to_string(path)?;
        let db: JsonDb = serde_json::from_str(&content)?;
        Ok(db)
    }

    /// Write DB to JSON file
    pub fn write(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
