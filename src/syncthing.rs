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

//let rsp = client
//    .post(&format!("{}/rest/config/folders", &config.url))
//    .header("X-API-Key", &config.token)
//    .send()
//    .await?
//    .text()
//    .await?;

/// Get a list of syncthing folders from a running syncthing instance via the REST API.
/// The syncthing crate is unmaintained and incomplete. We only need to get paths from it anyway,
/// so just parse the raw JSON. Use the [config
/// endpoint](https://docs.syncthing.net/rest/config.html)
pub async fn get_folders(config: &Config) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let rsp = match client
        .post(&format!("{}/rest/config/folders", &config.url))
        .header("X-API-Key", &config.token)
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
        folders.push(PathBuf::from(folder["path"].as_str().unwrap()));
    }
    Ok(folders)
}
