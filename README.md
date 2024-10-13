<div align="center">
<h1>kriger</h1>
</div>

![](.github/assets/banner.png)

An exploit farm for attack/defense (A/D) CTFs. This is the next-generation exploit farm based on the work done
in [angrepa](https://github.com/Cyberlandslaget/angrepa). Kriger is a free software licensed under [AGPL-3.0-only](LICENSE).

> [!WARNING]
> **kriger** was developed in the context of [ECSC2024](https://ecsc2024.it/) and may not be actively maintained. There are various TODOs still
> present in the software which may affect the usage in a production environment. Some parts of the documentation and the ADRs
> are slightly outdated or inaccurate.
>
> Contributions to the project are welcome.

## Documentation

- [User Guide](docs/user.md)
- [Writing Exploits](docs/exploits.md)
- [Architecture](docs/architecture.md)
- [Emergency](docs/emergency.md)
- [Debugging](docs/debugging.md)

## Development

A Linux or macOS environment is **highly** recommended. Windows users should consider using WSL.

### Prerequisites

- Rust Toolchain (see [Install Rust](https://www.rust-lang.org/tools/install))
- [Docker](https://docs.docker.com/engine/install/) with [Compose v2](https://docs.docker.com/compose/install/) *(or
  other container runtimes with support for Docker Compose files)*

### Running required services

| Service  | Port                      |
|----------|---------------------------|
| nats     | 4222 (NATS & JetStream)   |
| k3s      | 6443 (Kubernetes API)     |
| registry | 5000 (Container registry) |
| jeager   | 4317 (OTLP)               |
| jeager   | 16686 (Jeager UI)         |

**Start services:**

```bash
docker compose up -d --remove-orphans
export KUBECONFIG="$(pwd)/run/k3s/kubeconfig"
```

**Stop services:**

```bash
docker compose down
```

### Running kriger

Run the server components:

```bash
cargo r server # This will run the NATS migration for the first time
docker compose start nats-init # This will seed the K/V store with test data

cargo r server --single data/config/dev.toml 
```

Run the competition mock:

```bash
cargo r --bin kriger_mock -- --autotick 5
```

The mock will be available at port `:8080` by default.

Run the runner component:

```bash
RUST_LOG=debug cargo run runner --exploit test --service "Service 1 Checker 1" --flag-format "[A-Z0-9]{31}=" -- bash -c 'head -c 19 /dev/random | base32'
```

> **Note:** This is not required if the example exploit is deployed.

### Running the example exploit

Deploying the example exploit:

```bash
cd data/examples/python-test
cargo r deploy # or kriger deploy
```

## Terminologies

| Name            | Explanation                                                                                                                 |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------|
| Exploit         | A script or a program that exploits a vulnerable service to retrieve flags.                                                 |
| Execution       | A single run of an exploit. An execution will be run against the desired target.                                            |
| Team network ID | A publicly-known persistent ID associated with a team. The identity of the team isn't necessarily known or tied to this ID. |
