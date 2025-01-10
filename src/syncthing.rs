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
pub async fn get_folders(config: &Config) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    /// FIXME: make a class to avoid putting this on the heap
    let rsp = client
        .post(&format!("{}/rest/config/folders", &config.url))
        .header("X-API-Key", &config.token)
        .send()
        .await?
        .text()
        .await?;
    let rsp_json: serde_json::Value = serde_json::from_str(&rsp)?;
    let mut folders = Vec::new();
    for folder in rsp_json.as_array().unwrap() {
        folders.push(PathBuf::from(folder["path"].as_str().unwrap()));
    }
    Ok(folders)
}
