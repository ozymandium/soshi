use log::debug;
use serde::Deserialize;
use std::path::PathBuf;

use color_eyre::eyre::Result;
use gethostname::gethostname;
use ntfy::dispatcher::auth::Auth;
use ntfy::dispatcher::Dispatcher;
use ntfy::error::NtfyError as Error;
use ntfy::payload::{Payload, Priority};

/// Configuration for sending requests to ntfy.sh
#[derive(Debug, Deserialize)]
pub struct Config {
    /// URL of the ntfy.sh instance
    pub instance: String,
    /// Topic to send notifications to
    pub topic: String,
    /// Token for authenticating with the ntfy.sh instance
    pub token: String,
}

impl Config {
    pub fn clone(&self) -> Config {
        Config {
            instance: self.instance.clone(),
            topic: self.topic.clone(),
            token: self.token.clone(),
        }
    }
}

/// All ntfy.sh interaction is done through the Ntfy struct
pub struct Ntfy {
    /// Configuration needed to send requests to ntfy.sh
    config: Config,

    /// Hostname of the machine
    hostname: String,

    /// Sends payloads to ntfy.sh
    dispatcher: Dispatcher,
}

impl Ntfy {
    /// Create a new Ntfy instance
    ///
    /// # Arguments
    /// * `config`: configuration for the ntfy.sh instance
    ///
    /// # Returns
    /// A new Ntfy instance
    pub fn new(config: Config) -> Result<Ntfy, Error> {
        let hostname = gethostname().to_string_lossy().to_string();
        let dispatcher = Dispatcher::new(&config.instance, Some(Auth::token(&config.token)), None)?;
        Ok(Ntfy {
            config,
            hostname,
            dispatcher,
        })
    }

    /// Send a notification to ntfy.sh about new syncthing conflicts
    ///
    /// # Arguments
    /// * `conflicts`: list of new conflict files
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn conflicts(&self, conflicts: &[PathBuf]) -> Result<(), Error> {
        if conflicts.is_empty() {
            return Ok(());
        }
        let payload = Payload::new(&self.config.topic)
            .title(format!(
                "{}: {} new syncthing conflicts",
                &self.hostname,
                conflicts.len()
            ))
            .tags(["warning"])
            .message(
                conflicts
                    .iter()
                    .map(|p| format!("* `{}`", p.display()))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .markdown(true);
        self.dispatcher.send(&payload).await?;
        debug!("Sent notification");
        Ok(())
    }

    /// Send a notification to ntfy.sh about a soshi failure
    pub async fn failure(&self, message: &str) -> Result<(), Error> {
        let payload = Payload::new(&self.config.topic)
            .title(format!("{}: soshi failed", &self.hostname))
            .tags(["rotating_light"])
            .priority(Priority::High)
            .message(message)
            .markdown(false);
        self.dispatcher.send(&payload).await?;
        debug!("Sent notification");
        Ok(())
    }
}
