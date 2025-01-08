use regex::Regex;
use std::error::Error;
use std::path::PathBuf;

use soshi::json_db::JsonDb;

/// Struct for the config file
#[derive(Debug)] // TODO: add Deserialize when adding config file functionality
struct Config {
    // syncthing directories to search over
    st_dirs: Vec<PathBuf>,
    // path to the database file, which is a json list of conflict files
    db_path: PathBuf,
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
            let filename = path
                .file_name()
                .ok_or("No filename")?
                .to_str()
                .ok_or("No filename")?;
            if regex.is_match(filename) {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn main() -> Result<(), Box<dyn Error>> {
    // hardcode a config for now
    // TODO: add config file functionality
    let config = Config {
        st_dirs: vec![PathBuf::from("/home/roco/Documents")],
        db_path: PathBuf::from("/home/roco/src/soshi/conflicts.json"),
    };
    println!("{:?}", config);

    let old_conflicts = JsonDb::load(&config.db_path)?;

    // find current conflict files
    let cur_conflicts = find_conflicts(&config.st_dirs)?;

    // new conflicts are the difference between the current and old conflicts
    let new_conflicts: Vec<PathBuf> = cur_conflicts
        .iter()
        .filter(|path| !old_conflicts.conflicts.contains(path))
        .cloned()
        .collect();

    // print conflict files
    // TODO: add logging
    println!("New conflicts: {}", cur_conflicts.len());
    for file in new_conflicts {
        println!("    {}", file.display());
    }
    println!("Previous conflicts: {}", old_conflicts.conflicts.len());
    for file in old_conflicts.conflicts {
        println!("    {}", file.display());
    }

    // write new conflicts to the database file
    JsonDb::new(cur_conflicts).write(&config.db_path)?;

    Ok(())
}
