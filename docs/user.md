# User Guide

## Installing the CLI

### Installing the Rust toolchain

See [Install Rust](https://www.rust-lang.org/tools/install).

### Building and installing the CLI

Run the following command in the project's root directory:

```bash
cargo install --path crates/kriger --bin kriger
```

Alternatively, a prebuilt binary or the container image can be used.

## Configuring the CLI

### Alternative 1. Automatic Setup

TBD

### Alternative 2. Manual Setup

The config files are located at:

- Linux: `~/.config/kriger/cli.toml` (`$XDG_CONFIG_HOME`)
- macOS: `~/Library/Application Support/kriger/cli.toml`
- Windows: `C:\Users\...\AppData\Roaming\kriger\cli.toml`

Replace the required values and save the configuration file.

```toml
[registry]
secure = true
registry = "r.o99.no"
username = "user"
password = "[FILL IN HERE]"

[client]
rest_url = "https://kriger.o99.no/api"
ws_url = "wss://kriger.o99.no/ws"
```

Ensure that the permissions of the file is not globally-readable.

Changing the permissions on a **Unix-like system**:

```bash
chmod 400 "$XDG_CONFIG_HOME/kriger/cli.toml"
```

## Setting up Docker CLI

### Installing Docker

See [Docker's documentation](https://docs.docker.com/get-docker/) for how you can install Docker on your system. Mac
users may be interested in using [Colima](https://github.com/abiosoft/colima) instead.

After setting up Docker, log in to the container registry using the following commands:

```
docker login https://r.o99.no
```

## Next Steps

See _[Writing Exploits](exploits.md)_.
