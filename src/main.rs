use regex::Regex;
use std::path::PathBuf;
use serde::Deserialize;
use std::{
    fs,
    error::Error,
};

/// Struct for the config file
#[derive(Debug)] // TODO: add Deserialize when adding config file functionality
struct Config {
    // syncthing directories to search over
    st_dirs: Vec<PathBuf>,
    // path to the database file, which is a json list of conflict files
    db_path: PathBuf,
}

/// Struct for the json database file
#[derive(Deserialize)]
struct JsonDb {
    conflicts: Vec<PathBuf>,
}

impl JsonDb {
    /// Empty constructor for when the database file does not exist
    ///
    /// # Returns
    /// A new JsonDb with an empty list of conflicts
    fn does_not_exist() -> JsonDb {
        JsonDb { conflicts: Vec::new() }
    }
}

/// Finds syncthing conflict files in specified directories
///
/// # Arguments
/// * `st_dirs`: syncthing directories to search over
/// 
/// # Returns
fn find_conflicts(st_dirs: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let conflict_re = Regex::new(r".*\.sync-conflict-\d{8}-\d{6}-[0-9A-Z]{7}\..*")?;
    let mut conflicts = Vec::new();
    for dir in st_dirs {
        let found = find_files(dir, &conflict_re)?;
        for file in found {
            conflicts.push(file);
        }
    }
    Ok(conflicts)
}

/// Recursively finds files in a directory or its subdirectories that match a regex
///
/// # Arguments
/// * `dir`: directory to search in
/// * `regex`: regex to match files against
///
/// # Returns
/// A vector of paths to files that match the regex
fn find_files(dir: &PathBuf, regex: &Regex) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            //files.extend(find_files(&path, regex));
            let found = find_files(&path, regex)?;
            for file in found {
                files.push(file);
            }
        } else {
            let filename = path.file_name().ok_or("No filename")?.to_str().ok_or("No filename")?;
            if regex.is_match(filename) {
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// Loads the database file and returns a list of conflict files from previous runs. If the file
/// does not exist, returns an empty list.
///
/// # Arguments
/// * `db_path`: path to the database file
///
/// # Returns
/// Parsed json list of conflict files
fn load_db(db_path: &PathBuf) -> Result<JsonDb, Box<dyn Error>> {
    if !db_path.exists() {
        return Ok(JsonDb::does_not_exist());
    }
    let content = fs::read_to_string(db_path)?;
    let db: JsonDb = serde_json::from_str(&content)?;
    Ok(db)
}

fn main() -> Result<(), Box<dyn Error>> {
    
    // hardcode a config for now 
    // TODO: add config file functionality
    let config = Config {
        st_dirs: vec![PathBuf::from("/home/roco/Documents")],
        db_path: PathBuf::from("/home/roco/src/soshi/conflicts.json"),
    };
    println!("{:?}", config);

    // if db file exists, load it and set files found to be old conflicts. if not, set old
    // conflicts to empty
    if config.db_path.exists() {
        // load list of old conflicts from the json db file
        let old_conflicts = load_db(&config.db_path)?;
    } else {
        // set old conflicts to empty
    }

    // find current conflict files
    let cur_conflicts = find_conflicts(&config.st_dirs)?;

    // print conflict files
    // TODO: add logging
    println!("Found {} conflict files", cur_conflicts.len());
    for file in cur_conflicts {
        println!("{}", file.display());
    }
    Ok(())
}
