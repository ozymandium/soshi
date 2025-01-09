use serde::Deserialize;
use std::path::PathBuf;

use gethostname::gethostname;
use ntfy::dispatcher::auth::Auth;
use ntfy::dispatcher::Dispatcher;
use ntfy::error::NtfyError as Error;
use ntfy::payload::Payload;

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

/// Get a dispatcher for sending notifications
///
/// # Arguments
/// * `config`: configuration for the dispatcher
///
/// # Returns
/// A new dispatcher for sending notifications
pub fn get_dispatcher(config: &Config) -> Result<Dispatcher, Error> {
    Dispatcher::new(&config.instance, Some(Auth::token(&config.token)), None)
}

/// Send a notification to ntfy.sh
///
/// # Arguments
/// * `dispatcher`: dispatcher for sending notifications
/// * `config`: configuration for the dispatcher
/// * `new_conflicts`: list of new conflict files
///
/// # Returns
/// Result indicating success or failure
#[tokio::main]
pub async fn notify(
    dispatcher: &Dispatcher,
    config: &Config,
    new_conflicts: &[PathBuf],
) -> Result<(), Error> {
    let hostname_binding = gethostname();
    let hostname = hostname_binding.to_string_lossy();
    let payload = Payload::new(&config.topic)
        .title(format!(
            "{}: {} new syncthing conflicts",
            &hostname,
            new_conflicts.len()
        ))
        // list the new conflicts
        //.message(new_conflicts.iter().map(|p| p.to_string()).collect::<Vec<String>>().join("\n"))
        .message("")
        .markdown(true);
    //async_task::block_on(dispatcher.send(&payload))?;
    dispatcher.send(&payload).await?;
    Ok(())
}

/// All ntfy.sh interaction is done through the Ntfy struct
pub struct Ntfy {
    /// Take ownership of the ntfy.sh configuration
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
        let hostname_binding = gethostname();
        let hostname = hostname_binding.to_string_lossy().to_string();
        let dispatcher = Dispatcher::new(&config.instance, Some(Auth::token(&config.token)), None)?;
        Ok(Ntfy {
            config,
            hostname,
            dispatcher,
        })
    }

    /// Send a notification to ntfy.sh
    ///
    /// # Arguments
    /// * `new_conflicts`: list of new conflict files
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn notify_conflicts(&self, new_conflicts: &[PathBuf]) -> Result<(), Error> {
        let payload = Payload::new(&self.config.topic)
            .title(format!(
                "{}: {} new syncthing conflicts",
                &self.hostname,
                new_conflicts.len()
            ))
            // list the new conflicts
            //.message(new_conflicts.iter().map(|p| p.to_string()).collect::<Vec<String>>().join("\n"))
            .message("")
            .markdown(true);
        self.dispatcher.send(&payload).await?;
        Ok(())
    }
}
