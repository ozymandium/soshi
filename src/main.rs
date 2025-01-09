use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueHint};
use regex::Regex;
use serde::Deserialize;
use tokio::signal::unix::{signal, SignalKind};
use toml;
//use tokio::time::self as tokio_time;
use tokio::time as tokio_time;

use soshi::json_db::JsonDb;
use soshi::ntfy::Config as NtfyConfig;
//use soshi::ntfy::{get_dispatcher, notify};
use soshi::ntfy::Ntfy;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short, long, default_value = "~/.config/soshi.toml", value_hint = ValueHint::FilePath)]
    config: PathBuf,
}

/// Struct for the config file
#[derive(Debug, Deserialize)] // TODO: add Deserialize when adding config file functionality
struct Config {
    // How frequently to check for new conflicts
    #[serde(with = "humantime_serde")]
    interval: Duration,
    // syncthing directories to search over
    st_dirs: Vec<PathBuf>,
    // path to the database file, which is a json list of conflict files
    db_path: PathBuf,
    // ntfy.sh configuration.
    ntfy: NtfyConfig,
}

/// Implementation of the Config struct
impl Config {
    /// Load the configuration from a TOML file
    ///
    /// # Arguments
    /// * `path`: path to the TOML file
    ///
    /// # Returns
    /// The configuration struct
    pub fn load(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
        if !path.exists() {
            return Err(format!("Config file does not exist: {}", path.display()).into());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
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

/// Recursively finds files in a directory or its subdirectories that match a regex. Ignores
/// anything that is a symlink.
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
        let path = entry?.path();
        // ignore symlinks
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            //let found = find_files(&path, regex)?;
            //for file in found {
            //    files.push(file);
            //}
            files.append(&mut find_files(&path, regex)?);
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

async fn run(config: &Config, ntfy: &Ntfy) -> Result<(), Box<dyn Error>> {
    let old_db = JsonDb::load(&config.db_path)?;
    let cur_conflicts = find_conflicts(&config.st_dirs)?;
    // new conflicts are the difference between the current and old conflicts
    // if the db didn't exist, all current conflicts are new
    let new_conflicts: Vec<PathBuf> = cur_conflicts
        .iter()
        .filter(|path| !old_db.conflicts.contains(path))
        .cloned()
        .collect();

    println!("Previous conflicts: {}", old_db.conflicts.len());
    for file in old_db.conflicts {
        println!("    {}", file.display());
    }
    println!("New conflicts: {}", new_conflicts.len());
    for file in &new_conflicts {
        println!("    {}", file.display());
    }
    // write new conflicts to the database file
    JsonDb::new(cur_conflicts).write(&config.db_path)?;

    ntfy.conflicts(&new_conflicts).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = Config::load(&args.config)?;
    println!("Config: {:?}", config);

    //let dispatcher = get_dispatcher(&config.ntfy)?;
    let ntfy = Ntfy::new(config.ntfy.clone())?;

    let mut stream_sigterm = signal(SignalKind::terminate())?;

    loop {
        if let Err(e) = run(&config, &ntfy).await {
            return Err(e);
        }
        let delay = tokio_time::sleep(config.interval);
        tokio::select! {
            _ = delay => {},
            _ = stream_sigterm.recv() => {
                println!("Received SIGTERM");
                break;
            },
        }
    }
    Ok(())
}
