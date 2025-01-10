use expanduser::expanduser;
use log::debug;
use reqwest;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;

/// Configuration for interacting with a syncthing instance REST API
#[derive(Debug, Deserialize)]
pub struct Config {
    /// URL of the syncthing instance, including port
    url: String,
    /// API key for the syncthing instance
    token: String,
}

/// Get a list of syncthing folders from a running syncthing instance via the REST API.
/// The syncthing crate is unmaintained and incomplete. We only need to get paths from it anyway,
/// so just parse the raw JSON. Use the [config
/// endpoint](https://docs.syncthing.net/rest/config.html)
///
/// # Arguments
/// * `config`: configuration for the syncthing instance
///
/// # Returns
/// A list of paths to syncthing folders which are valid directories
pub async fn get_folders(config: &Config) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let rsp = match client
        .get(&format!("{}/rest/config/folders", &config.url))
        .header("Authorization", format!("Bearer {}", &config.token))
        .send()
        .await
    {
        Ok(rsp) => rsp,
        Err(e) => return Err(format!("Error sending request to syncthing:\n{}", e).into()),
    };
    let rsp_text = match rsp.text().await {
        Ok(rsp_text) => rsp_text,
        Err(e) => return Err(format!("Error reading response from syncthing:\n{}", e).into()),
    };
    let rsp_json: serde_json::Value = match serde_json::from_str(&rsp_text) {
        Ok(rsp_json) => rsp_json,
        Err(e) => {
            return Err(format!(
                "Error parsing JSON from syncthing:\n{}\nText:\n{}",
                e, rsp_text
            )
            .into())
        }
    };
    let mut folders = Vec::new();
    for folder in rsp_json.as_array().unwrap() {
        //let path = PathBuf::from(expanduser(folder["path"].as_str().unwrap()));
        let path_str = match folder["path"].as_str() {
            Some(path_str) => path_str,
            None => {
                return Err(format!(
                    "Error parsing folder path from syncthing response:\n{}",
                    rsp_text
                )
                .into())
            }
        };
        let expanded_path_str = match expanduser(path_str) {
            Ok(expanded_path_str) => expanded_path_str,
            Err(e) => {
                return Err(format!(
                    "Error expanding user in folder path from syncthing response:\n{}",
                    e
                )
                .into())
            }
        };
        let path = PathBuf::from(expanded_path_str);
        if path.exists() && path.is_dir() {
            folders.push(path);
        } else {
            return Err(format!(
                "Folder path does not exist or is not a directory: {}",
                path.display()
            )
            .into());
        }
    }
    debug!("Syncthing folders: {:?}", folders);
    Ok(folders)
}
