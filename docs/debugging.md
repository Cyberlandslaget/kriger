# Debugging

## Messaging / NATS

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
  "tick_start": 0,
  "flag_validity": 5,
  "flag_format": "FLAG_[^ ]+"
}
```

```bash
nats kv put config competition '{
  "start": "2024-01-01T08:00:00Z",
  "tick": 5,
  "tick_start": 0,
  "flag_validity": 5,
  "flag_format": "FLAG_[^ ]+"
}'
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
      "cpu_limit": "1",
      "mem_limit": "512M"
    }
  },
  "image": "r.o99.no/kriger-exploits/test"
}
```

```bash
nats kv put exploits test '{"manifest":{"name":"test","service":"Service 1 Checker 1","replicas":4,"enabled":true,"resources":{"cpu_limit":"1","mem_limit":"512M"}},"image":"r.o99.no/kriger-exploits/test"}'
```

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
