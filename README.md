soshi
===

A simple monitor for [Syncthing](https://syncthing.net) conflicts.
When new conflicts are detected, soshi sends [ntfy](https://ntfy.sh) notifications.

Currently only supports Linux.

## Configuration

- Most configuration is done via configuration file. 
    - By default, soshi looks for it at `~/.config/soshi.toml`.
    - Pass `-c <location>` via CLI to change this.
- Logging is configured by setting the `RUST_LOG` environment variable. Since this is meant to be run by systemd, timestamps are omitted from logging.
- 

Configuration file example:
```toml
interval = "1hr"
db_path = "~/.local/state/soshi.json"
[syncthing]
    url = "http://localhost:8384"
    token = "your_syncthing_token" # get this from the webui admin panel
[ntfy]
    instance = "https://ntfy.sh"
    topic = "your_topic"
    token = "your_ntfy_token"
```
