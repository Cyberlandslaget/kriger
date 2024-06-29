# kriger

An exploit farm for attack/defense (A/D) CTFs. This is the next-generation exploit farm based on the work done
in [angrepa](https://github.com/Cyberlandslaget/angrepa).

![](.github/assets/logo.png)

## Components

- **kriger**: A meta-package containing all server components.
    - **kriger-controller**: Responsible for retrieving teams.json (attack data), scheduling exploit runs and
      provisioning compute for the exploit runners.
    - **kriger-rest**: REST API for the CLI and the web frontend.
    - **kriger-ws**: WebSocket server to send real-time data to the web frontend or other consumers.
    - **kriger-runner**: Responsible for executing the exploits.
    - **kriger-submitter**: Responsible for submitting flags to the competition system.
    - **kriger-metrics**: [OpenMetrics](https://openmetrics.io/)/[Prometheus](https://prometheus.io/)-compatible
      metrics exporter.
- **kriger-cli**: The command line interface (CLI) used to create, test, and deploy exploits.

