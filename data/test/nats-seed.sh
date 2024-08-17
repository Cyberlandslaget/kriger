#!/usr/bin/env sh

nats kv put config competition '{
  "start": "2024-01-01T08:00:00Z",
  "tick": 5,
  "tick_start": 0,
  "flag_validity": 5,
  "flag_format": "[A-Z0-9]{31}=",
  "submitter": {
    "type": "dummy",
    "interval": 1
  },
  "fetcher": {
    "type": "dummy"
  }
}'

# TODO: Remove once the fetcher is functional
nats kv put services U2VydmljZSAxIENoZWNrZXIgMQ '{"name": "Service 1 Checker 1", "has_hint": false}'

for i in `seq 0 9`
do
  nats kv put teams "$i" "{\"name\": \"Team $i\", \"ip_address\": \"10.60.$i.1\", \"services\":{}}"
done
