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

## Setting up Docker CLI

### Installing Docker

See [Docker's documentation](https://docs.docker.com/get-docker/) for how you can install Docker on your system. Mac
users may be interested in using [Colima](https://github.com/abiosoft/colima) instead.

After setting up Docker, log in to the container registry using the following commands:

```
docker login https://r.o99.no 
```

## Next Steps

See *[Writing Exploits](exploits.md)*.

