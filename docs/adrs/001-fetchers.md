# Fetchers

## Status

Draft

## Context

There are varying requirements for various attack/defense CTFs regarding the data needed to perform a successful attack
against the services running on the target vulnbox.

Examples:

- **CINI (ECSC 2024)** provides a "flag ids" endpoint which must be queried by at least one of the following: service,
  team id. This means that the fetcher for CINI must fetch the list of teams and/or the list of services before querying
  the "flag ids".
- **FAUST CTF** provides a "flag ids" endpoint which returns a list of teams, services, and the "flag ids" associated
  with them respectively. An example of a "flag id" is a username of a user that's registered on a service, which is
  required or highly recommended to use for an exploit execution.

Furthermore, there are other considerations to make when fetching data from various attack/defense CTFs. The following
should be considered (non-exhaustive):

- A "team id" may refer to a persistent ID that's associated with a team. However, the association between the ID and
  the team identity may not be known. A "team id" may only be used to construct the target's IP address based on a
  format. Some attack/defense CTFs choose to anonymize this to avoid targeting.
- Each team may have different IP addresses for each service.
- Different kinds of data may have to be fetched separately.
- Service IP addresses may not follow a specific format that's dependent on the team id.

## Decision

Fetcher implementations should have the flexibility required while maintaining an efficient system.

## Consequences

The fetcher trait is generalized and does not enforce a strongly opinionated pattern for the data that it must return.
The fetcher is executed on an interval and is responsible for persisting the data that it retrieved.
