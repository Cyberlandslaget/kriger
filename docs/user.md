# User Guide

- [Prerequisites](#prerequisites)
- [Installing the CLI](#installing-the-cli)
- [Configuring the CLI](#configuring-the-cli)
- [Setting up Docker CLI](#setting-up-docker-cli)

## Prerequisites

### Required

- Rust toolchain - see [Install Rust](https://www.rust-lang.org/tools/install).

### Optional, but recommended

- [**uv**](https://docs.astral.sh/uv/getting-started/installation/) - an extremely fast Python package
  manager ([10-100x faster than pip](https://github.com/astral-sh/uv/blob/main/BENCHMARKS.md)). The performance
  improvement will be taken advantage of when developing exploits.

## Installing the CLI

### Alternative 1: Building and installing the CLI

Run the following command in the project's root directory:

```bash
cargo install --path crates/kriger --bin kriger
```

### Alternative 2: Using a prebuilt binary

Simply copy the prebuilt binary to a directory that is included in your shell's path. On Linux
distributions, `~/.local/bin` can be used.

### Alternative 3. Using the container image

TODO

## Configuring the CLI

### Alternative 1: Automatic Setup

TBD

### Alternative 2: Manual Setup

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

> [!NOTE]
> The configuration file name is `cli.dev.toml`
> when [debug_assertions](https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions) are
> enabled. In practice, this means that `cli.dev.toml` will be used during local development.

> [!IMPORTANT]
> Ensure that the file is not globally-readable as it contains the registry credentials.
>
> Changing the file permissions on a **Linux distribution**:
>
> ```bash
> chmod 400 "$XDG_CONFIG_HOME/kriger/cli.toml"
> ```

## Setting up Docker CLI

### Installing Docker

See [Docker's documentation](https://docs.docker.com/get-docker/) for how you can install Docker on your system. Mac
users may be interested in using [Colima](https://github.com/abiosoft/colima) instead.

> [!IMPORTANT]
> Ensure that your current user has access to the Docker daemon.
> On most **Linux distributions**, adding the user to the `docker`
> group will be sufficient:
>
> ```bash
> sudo useradd -aG docker `whoami`
> ```

After setting up Docker, log in to the container registry using the following commands:

```bash
docker login https://r.o99.no
```

## Next Steps

See _[Writing Exploits](exploits.md)_.
