# kriger

An exploit farm for attack/defense (A/D) CTFs. This is the next-generation exploit farm based on the work done
in [angrepa](https://github.com/Cyberlandslaget/angrepa).

![](.github/assets/logo.png)

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
cargo run server ---single
```

Run the runner component:

```bash
RUST_LOG=debug cargo run runner --exploit test -- bash -c 'echo FLAG_$(openssl rand -base64 18)'
```

### Building Images Locally

Run the following command in the project's root directory:

```bash
docker build \
  -t r.o99.no/kriger/kriger  \
  -t localhost:5000/kriger/kriger \
  .
```

Build base images:

```bash
docker build --build-arg "REGISTRY=localhost:5000" \
  -t r.o99.no/kriger/exploit-base:python  \
  -t localhost:5000/kriger/exploit-base:python \
  data/base/python
```

Build templates as exploits:

```bash
tar -ch -C data/templates/python . | docker build --build-arg "REGISTRY=localhost:5000" \
  -t r.o99.no/kriger-exploits/test \
  -t localhost:5000/kriger-exploits/test \
  -
```

> **Note:** `tar -ch` is required to archive the build context since symlinks are used in the templates.

Push the exploit to the registry:
```bash
docker push localhost:5000/kriger-exploits/test
```

Run the exploit directly:

```bash
docker run --rm -it --network kriger_default -e EXPLOIT=test r.o99.no/kriger-exploits/test
```

## Terminologies

| Name            | Explanation                                                                                                                 |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------|
| Exploit         | A script or a program that exploits a vulnerable service to retrieve flags.                                                 |
| Execution       | A single run of an exploit. An execution will be run against the desired target.                                            |
| Team network ID | A publicly-known persistent ID associated with a team. The identity of the team isn't necessarily known or tied to this ID. |
