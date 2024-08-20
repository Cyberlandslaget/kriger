# Debugging

## Messaging / NATS

This document highlights manual testing operations for NATS. Do note that this should only be required during
development.

### Local Debugging

This assumes that the [development guide](../README.md#development) was followed.

```bash
docker run --network kriger_default -e NATS_URL=nats://nats:4222 --rm -it natsio/nats-box
```

> **Note**: Check `docker network ls` if the network name of the Docker compose project is different from the provided
> command.

### Sending Messages Manually

#### Competition Config

```json
{
  "start": "2024-01-01T08:00:00Z",
  "tick": 5,
  "tickStart": 0,
  "flagValidity": 5,
  "flagFormat": "[A-Z0-9]{31}=",
  "submitter": {
    "type": "dummy",
    "interval": 1
  },
  "fetcher": {
    "type": "dummy"
  }
}
```

```bash
nats kv put config competition '{"start": "2024-01-01T08:00:00Z", "tick": 5, "tickStart": 0, "flagValidity": 5, "flagFormat": "[A-Z0-9]{31}=", "submitter": {"type": "dummy", "interval": 1}, "fetcher": {"type": "dummy"}}'
```

#### Scheduler Testing

Team:

```json
{
  "ipAddress": "127.0.0.1",
  "services": {}
}
```

```bash
nats kv put teams 1 '{"ipAddress": "127.0.0.1", "services":{}}'
```

Service:

```json
{
  "name": "service 1 Checker 1",
  "hasHint": false
}
```

```bash
nats kv put services U2VydmljZSAxIENoZWNrZXIgMQ '{"name": "service 1 Checker 1", "hasHint": false}'
```

#### Controller Testing

```json
{
  "manifest": {
    "name": "test",
    "service": "Service 1 Checker 1",
    "replicas": 4,
    "enabled": true,
    "resources": {
      "cpuLimit": "1",
      "memLimit": "512M"
    }
  },
  "image": "r.o99.no/kriger-exploits/test"
}
```

```bash
nats kv put exploits test '{"manifest":{"name":"test","service":"Service 1 Checker 1","replicas":4,"enabled":true,"resources":{"cpuLimit":"1","memLimit":"512M"}},"image":"r.o99.no/kriger-exploits/test"}'
```

### Submitter Testing

```bash
flag=$(head -c 19 /dev/random | base32) && nats kv put flags "$(echo $flag | base64)".submit "{\"f\":\"$flag\"}"
````

#### Runner Testing

```json5
{
  // The target's IP address
  "a": "127.0.0.1",
  // Optional hint
  "h": {}
}
```

```bash
nats pub executions.test.request --count 1 '{"a":"127.0.0.1","h":{}}'
```

## Kubernetes

> **Tips:** Tools like [k9s](https://github.com/derailed/k9s) will make it easier to manage the Kubernetes "cluster".

## Containers

Building an exploit container image manually:

```bash
tar -ch -C data/examples/python-test . | docker build --build-arg "REGISTRY=localhost:5000" \
  -t r.o99.no/kriger-exploits/test \
  -t localhost:5000/kriger-exploits/test \
  -
```

> **Note:** `tar -ch` is required to archive the build context since symlinks are used in the templates.

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

Run the exploit directly:

```bash
docker run --rm -it --network kriger_default -e EXPLOIT=test r.o99.no/kriger-exploits/test
```
