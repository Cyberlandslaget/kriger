#!/usr/bin/env sh

# TODO: Remove once the fetcher is functional
nats kv put services U2VydmljZSAxIENoZWNrZXIgMQ '{"name": "Service 1 Checker 1", "hasHint": false}'

for i in `seq 0 9`
do
  nats kv put teams "$i" "{\"name\": \"Team $i\", \"ipAddress\": \"10.60.$i.1\", \"services\":{}}"
done
