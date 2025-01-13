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
- See the configuration [example](./examples/config.toml).
