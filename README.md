# kriger

An exploit farm for attack/defense (A/D) CTFs. This is the next-generation exploit farm based on the work done
in [angrepa](https://github.com/Cyberlandslaget/angrepa).

![](.github/assets/logo.png)

## Documentation

- [User Guide](docs/user.md)
- [Writing Exploits](docs/exploits.md)
- [Architecture](docs/architecture.md)

## Development

A Linux or macOS environment is **highly** recommended. Windows users should consider using WSL.

### Prerequisites

- Rust Toolchain (see [Install Rust](https://www.rust-lang.org/tools/install))
- [Docker](https://docs.docker.com/engine/install/) with [Compose v2](https://docs.docker.com/compose/install/) *(or
  other container runtimes with support for Docker Compose files)*

### Running required services

| Service | Port                    |
|---------|-------------------------|
| nats    | 4222 (NATS & JetStream) |
| k3s     | 6443 (Kubernetes API)   |

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

```bash
cargo run
```

### Debugging

**Nats debugging:**

```bash
docker run --network kriger_default -e NATS_URL=nats://nats:4222 --rm -it natsio/nats-box
```

> **Note**: Check `docker network ls` if the network name of the Docker compose project is different from the provided
> command.

**Kubernetes (k3s):**

```bash
kubectl get nodes
```

> **Tips:** Tools like [k9s](https://github.com/derailed/k9s) will make it easier to manage the Kubernetes "cluster".

## Terminologies

| Name            | Explanation                                                                                                                 |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------|
| Exploit         | A script or a program that exploits a vulnerable service to retrieve flags.                                                 |
| Execution       | A single run of an exploit. An execution will be run against the desired target.                                            |
| Team network ID | A publicly-known persistent ID associated with a team. The identity of the team isn't necessarily known or tied to this ID. |
