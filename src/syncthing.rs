use color_eyre::eyre::{eyre, Result, WrapErr};
use expanduser::expanduser;
use log::debug;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest;
use serde::Deserialize;
use std::path::PathBuf;

/// Recursively finds files in a directory or its subdirectories that match a regex. Ignores
/// anything that is a symlink. This is a general purpose function that can be used for any
/// file search.
///
/// # Arguments
/// * `dir`: directory to search in
/// * `regex`: regex to match files against
///
/// # Returns
/// A vector of paths to files that match the regex
fn find_files(dir: &PathBuf, regex: &Regex) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in dir.read_dir()? {
        let path = entry?.path();
        if path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            files.append(&mut find_files(&path, regex)?);
        } else {
            let filename = path
                .file_name()
                .ok_or(eyre!("No filename"))?
                .to_str()
                .ok_or(eyre!("No filename"))?;
            if regex.is_match(filename) {
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// Configuration for interacting with a syncthing instance REST API
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// URL of the syncthing instance, including port if not the default HTTP port.
    url: String,
    /// API key for the syncthing instance. Get this from the GUI.
    token: String,
}

/// All syncthing interaction is done through the Syncthing struct
pub struct Syncthing {
    /// Configuration needed to interact with syncthing
    config: Config,
    /// Client for sending requests to syncthing
    client: reqwest::Client,
}

/// Finds syncthing conflict files in specified directories.
/// Conflict files look like `filename.sync-conflict-20210101-123456-ABC1234.ext`
/// for a conflict with:
/// * file `filename.ext`
/// * on machine id `ABC1234`
/// * on day `2021-01-01`
/// * at time `12:34:56`.
static CONFLICT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r".*\.sync-conflict-\d{8}-\d{6}-[0-9A-Z]{7}\..*").unwrap());

impl Syncthing {
    pub fn new(config: &Config) -> Syncthing {
        Syncthing {
            config: config.clone(),
            client: reqwest::Client::new(),
        }
    }

    /// Builds a request for the folders endpoint with the authorization header. Reuse to get folders.
    ///
    /// # Returns
    /// Request for the folders endpoint with the authorization header.
    fn build_req(&self) -> reqwest::RequestBuilder {
        self.client
            .get(&format!("{}/rest/config/folders", &self.config.url))
            .header("Authorization", format!("Bearer {}", &self.config.token))
    }

    async fn get_folders(&self) -> Result<Vec<PathBuf>> {
        let rsp = self
            .build_req()
            .send()
            .await
            .wrap_err("Sending request to syncthing")?;
        let rsp_text = rsp
            .text()
            .await
            .wrap_err("Reading response from syncthing")?;
        let rsp_json: serde_json::Value = serde_json::from_str(&rsp_text).wrap_err(format!(
            "Parsing JSON from syncthing. Text that failed parsing:\n{}",
            rsp_text
        ))?;
        let mut folders = Vec::new();
        for folder in rsp_json.as_array().unwrap() {
            let path_str = folder["path"].as_str().ok_or(eyre!("No path"))?;
            let expanded_path_str = expanduser(path_str).unwrap();
            let path = PathBuf::from(expanded_path_str);
            if path.exists() && path.is_dir() {
                folders.push(path);
            } else {
                return Err(eyre!(
                    "Path does not exist or is not a directory: {}",
                    path.display()
                ));
            }
        }
        debug!("Syncthing folders: {:?}", folders);
        Ok(folders)
    }

    /// Finds syncthing conflict files in specified directories
    ///
    /// # Arguments
    /// * `folders`: syncthing directories to search over
    ///
    /// # Returns
    /// A vector of paths to conflict files
    pub async fn conflicts(&self) -> Result<Vec<PathBuf>> {
        let mut conflicts = Vec::new();
        let folders = self.get_folders().await?;
        for dir in folders {
            let found = find_files(&dir, &CONFLICT_RE).wrap_err(format!(
                "Finding syncthing conflicts in directory: {}",
                dir.display()
            ))?;
            for file in found {
                conflicts.push(file);
            }
        }
        Ok(conflicts)
    }
}
