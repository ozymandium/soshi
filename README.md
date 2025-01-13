soshi
===

A simple monitor for [Syncthing](https://syncthing.net) conflicts.
When new conflicts are detected, soshi sends [ntfy](https://ntfy.sh) notifications.

Queries a configurable syncthing endpoint at fixed intervals to retrieve a list of synced folders. 
Adding new folders to syncthing does not require reconfiguring/restarting soshi.

Currently only tested on Linux. 
However, since soshi is currently setup to do manual scans at fixed intervals, running on exotic filesystems or other OSes should take little/no effort.

## Configuration

- Most configuration is done via configuration file. 
    - By default, soshi looks for it at `~/.config/soshi.toml`.
    - Pass `-c <location>` via CLI to change this.
- Logging is configured by setting the `RUST_LOG` environment variable. Since this is meant to be run by systemd, timestamps are omitted from logging.
- See the configuration [example](./examples/config.toml).
