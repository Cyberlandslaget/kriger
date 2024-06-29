# kriger

An exploit farm for attack/defense (A/D) CTFs. This is the next-generation exploit farm based on the work done
in [angrepa](https://github.com/Cyberlandslaget/angrepa).

![](.github/assets/logo.png)

## Components

- **kriger**: A meta-package containing all server components.
    - **kriger-controller**: Responsible for scheduling exploit runs and provisioning compute for the exploit runners.
    - **kriger-fetcher**: Responsible for fetching various data from the competition system, for example teams.json. The
      fetcher will also persist the received data.
    - **kriger-metrics**: [OpenMetrics](https://openmetrics.io/)/[Prometheus](https://prometheus.io/)-compatible
    - **kriger-rest**: REST API for the CLI and the web frontend.
    - **kriger-runner**: Responsible for executing the exploits.
    - **kriger-submitter**: Responsible for submitting flags to the competition system.
      metrics exporter.
    - **kriger-ws**: WebSocket server to send real-time data to the web frontend or other consumers.
- **kriger-cli**: The command line interface (CLI) used to create, test, and deploy exploits.

### Component topology

**Server components**:

| Component             | Requirements             | Replicas                 |
|-----------------------|--------------------------|--------------------------|
| **kriger-controller** | Nats, Kubernetes API     | Exactly one              |
| **kriger-fetcher**    | Nats, Competition system | At least one             |                 
| **kriger-metrics**    | Nats                     | At least one / any       |                 
| **kriger-rest**       | Nats                     | At least one / any       |
| **kriger-runner**     | Nats, Competition system | At least one per exploit |                 
| **kriger-submitter**  | Nats, Competition system | At least one             |                 
| **kriger-ws**         | Nats                     | At least one / any       | 

**Client components**:

| Component           | Requirements           | Replicas |
|---------------------|------------------------|----------|
| **kriger-cli**      | kriger-rest, kriger-ws | Any      |                 
| **kriger-frontend** | kriger-rest, kriger-ws | Any      |                 

Replica counts marked with *any* means that the component is deemed to be non-critical for the exploit farm to function.
