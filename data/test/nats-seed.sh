#!/usr/bin/env sh

# SPDX-License-Identifier: AGPL-3.0-only
# Copyright Authors of kriger

# TODO: Remove once the fetcher is functional
nats kv put services U2VydmljZSAxIENoZWNrZXIgMQ '{"name": "Service 1 Checker 1", "hasHint": false}'

for i in `seq 0 9`
do
  nats kv put teams "$i" "{\"name\": \"Team $i\", \"ipAddress\": \"10.60.$i.1\", \"services\":{}}"
done
