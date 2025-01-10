use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueHint};
use color_eyre::eyre::{eyre, Result};
use env_logger::{Builder as LogBuilder, Env as LogEnv};
use log::{debug, info, warn};
use serde::Deserialize;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time as tokio_time;
use toml;

use soshi::json_db::JsonDb;
use soshi::ntfy::Config as NtfyConfig;
use soshi::ntfy::Ntfy;
use soshi::syncthing::{Config as SyncthingConfig, Syncthing};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the TOML configuration file
    #[arg(short='c', long, default_value = "~/.config/soshi.toml", value_hint = ValueHint::FilePath)]
    config: PathBuf,
}

/// Struct for the config file
#[derive(Debug, Deserialize)] // TODO: add Deserialize when adding config file functionality
struct Config {
    // How frequently to check for new conflicts
    #[serde(with = "humantime_serde")]
    interval: Duration,
    // syncthing instance url (with port)
    syncthing: SyncthingConfig,
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
    pub fn load(path: &PathBuf) -> Result<Config> {
        if !path.exists() {
            return Err(eyre!("Config file does not exist: {}", path.display()));
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Configure the logging system with env_logger. Call this function at the beginning of main.
fn setup_logging() {
    // allows setting the RUST_LOG environment variable to control logging
    LogBuilder::from_env(LogEnv::default())
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            writeln!(buf, "{level_style}{}{level_style:#}", record.args())
        })
        .init();
}

fn log_conflicts(old: &[PathBuf], cur: &[PathBuf], new: &[PathBuf], res: &[PathBuf]) {
    debug!("Old conflicts: {}", old.len());
    for file in old {
        debug!("    * {}", file.display());
    }
    debug!("Current conflicts: {}", cur.len());
    for file in cur {
        debug!("    * {}", file.display());
    }
    if new.is_empty() {
        debug!("New conflicts: 0");
    } else {
        warn!("New conflicts: {}", new.len());
        for file in new {
            warn!("    * {}", file.display());
        }
    }
    if res.is_empty() {
        debug!("Resolved conflicts: 0");
    } else {
        info!("Resolved conflicts: {}", res.len());
        for file in res {
            info!("    * {}", file.display());
        }
    }
}

async fn run(config: &Config, syncthing: &Syncthing, ntfy: &Ntfy) -> Result<()> {
    let old_db = JsonDb::load(&config.db_path)?;
    let cur_conflicts = syncthing.conflicts().await?;
    // new conflicts are the difference between the current and old conflicts
    // if the db didn't exist, all current conflicts are new
    let new_conflicts: Vec<PathBuf> = cur_conflicts
        .iter()
        .filter(|path| !old_db.conflicts.contains(path))
        .cloned()
        .collect();
    // resolved conflicts are the difference between the old and current conflicts
    let res_conflicts: Vec<PathBuf> = old_db
        .conflicts
        .iter()
        .filter(|path| !cur_conflicts.contains(path))
        .cloned()
        .collect();
    log_conflicts(
        &old_db.conflicts,
        &cur_conflicts,
        &new_conflicts,
        &res_conflicts,
    );
    // write new conflicts to the database file
    JsonDb::new(cur_conflicts).write(&config.db_path)?;
    ntfy.conflicts(&new_conflicts).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logging();

    let args = Args::parse();

    // load config file
    let config = match Config::load(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config file:\n{}", e);
            return Err(e);
        }
    };
    debug!("Config: {:?}", config);

    let syncthing = Syncthing::new(&config.syncthing);
    let ntfy = Ntfy::new(config.ntfy.clone())?;

    let mut stream_sigterm = signal(SignalKind::terminate())?;
    loop {
        if let Err(e) = run(&config, &syncthing, &ntfy).await {
            eprintln!("Error running main loop:\n{}", e);
            return Err(e);
        }
        let delay = tokio_time::sleep(config.interval);
        tokio::select! {
            _ = delay => {},
            _ = stream_sigterm.recv() => {
                debug!("Received SIGTERM");
                break;
            },
        }
    }
    Ok(())
}
