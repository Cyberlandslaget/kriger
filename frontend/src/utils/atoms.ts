import { atom } from "jotai";
import type {
  ExecutionRequestMessage,
  ExecutionResultMessage,
  Exploit,
  FlagSubmissionMessage,
  FlagSubmissionResultMessage,
  ServerConfig,
  Service,
  Team,
} from "../services/models";
import type { TeamExecutionMap, TeamFlagMap } from "./types";
import { FlagCode } from "./enums";

export const serverConfigAtom = atom<ServerConfig | undefined>(undefined);

export const statusAtom = atom({
  currentTick: -1,
});

export const currentTickAtom = atom(
  (get) => get(statusAtom).currentTick,
  (_get, set, currentTick: number) =>
    set(statusAtom, (current) => ({
      ...current,
      currentTick,
    })),
);

export const exploitsAtom = atom<Exploit[] | null>(null);
export const teamExecutionsAtom = atom<TeamExecutionMap>({});

export const teamsAtom = atom<Record<string, Team>>({});
export const servicesAtom = atom<Service[]>([]);

export const teamFlagStatusAtom = atom<TeamFlagMap>({});

export const serviceExploitAssociateAtom = atom((get) => {
  const exploits = get(exploitsAtom);

  const map = new Map<string, Exploit[]>();
  if (!exploits) return map;

  for (const exploit of exploits) {
    map.set(exploit.manifest.service, [
      ...(map.get(exploit.manifest.service) ?? []),
      exploit,
    ]);
  }

  return map;
});

export const exploitServiceAssociateAtom = atom((get) => {
  const exploits = get(exploitsAtom);

  const map = new Map<string, string>();
  if (!exploits) return map;

  for (const exploit of exploits) {
    map.set(exploit.manifest.name, exploit.manifest.service);
  }

  return map;
});

export const teamFlagSubmissionDispatch = atom(
  null,
  (_get, set, message: FlagSubmissionMessage | FlagSubmissionResultMessage) => {
    set(teamFlagStatusAtom, (prev) => {
      if (!message.teamId || !message.service) {
        return prev;
      }
      const prevStatus =
        prev[message.teamId]?.[message.service]?.[message.flag];

      if (prevStatus && !("status" in message)) {
        return prev;
      }

      return {
        ...prev,
        [message.teamId]: {
          ...prev[message.teamId],
          [message.service]: {
            ...prev[message.teamId]?.[message.service],
            [message.flag]: {
              // Pending SHOULD NOT overide any other statuses
              status:
                // If the message is a submission result message and if either:
                // - status was not defined previously
                // - the message's status has a higher precedence over the previous status
                "status" in message &&
                (!prevStatus?.status || message.status < prevStatus?.status)
                  ? message.status
                  : prevStatus?.status,
              // Keep the timestamp of when the flag was originally submitted.
              published: prevStatus?.published
                ? Math.min(prevStatus.published, message.published)
                : message.published,
              exploit: message.exploit ?? prevStatus.exploit,
            },
          },
        },
      };
    });
  },
);

export const teamFlagPurgeDispatch = atom(null, (_get, set, oldest: number) => {
  set(teamFlagStatusAtom, (current) => {
    return Object.fromEntries(
      Object.entries(current).map(([teamId, teamServiceMap]) => [
        teamId,
        Object.fromEntries(
          Object.entries(teamServiceMap).map(([service, flags]) => [
            service,
            Object.fromEntries(
              Object.entries(flags).filter(
                ([_, entry]) => entry.published >= oldest,
              ),
            ),
          ]),
        ),
      ]),
    );
  });
});

// TODO: Add tiered caching? We are aggregating everything every time 'teamFlagStatusAtom' updates.
// Premature optimization here can lead to inconsistent aggregation. We have to deeal with status updates,
// purging and the message delivery order.
export const flagStatusAggregateAtom = atom((get) => {
  const flagStatus = get(teamFlagStatusAtom);

  let count = 0;
  const statusMap = new Map<FlagCode, number>();
  const exploitCountMap: Map<string, number> = new Map();

  // We probably don't want to do FP here to avoid a lot of extra allocations
  for (const [_, serviceMap] of Object.entries(flagStatus)) {
    for (const [_, serviceFlags] of Object.entries(serviceMap)) {
      for (const [_, status] of Object.entries(serviceFlags)) {
        const key = status.status ?? FlagCode.Pending;
        statusMap.set(key, (statusMap.get(key) ?? 0) + 1);

        // Do counting per exploits
        if (status.exploit) {
          exploitCountMap.set(
            status.exploit,
            (exploitCountMap.get(status.exploit) ?? 0) + 1,
          );
        }
        ++count;
      }
    }
  }

  return {
    count,
    statusMap,
    exploitCountMap,
  };
});

export const exploitExecutionRequestDispatch = atom(
  null,
  (_get, set, message: ExecutionRequestMessage) => {
    set(teamExecutionsAtom, (prev) => {
      if (!message.teamId || !message.exploitName) {
        return prev;
      }

      const prevStatus =
        prev[message.exploitName]?.[message.teamId]?.[message.sequence];

      return {
        ...prev,
        [message.teamId]: {
          ...prev[message.teamId],
          [message.exploitName]: {
            ...prev[message.teamId]?.[message.exploitName],
            [message.sequence]: {
              ...prevStatus,
              published: message.published,
            },
          },
        },
      };
    });
  },
);

export const exploitExecutionResultDispatch = atom(
  null,
  (_get, set, message: ExecutionResultMessage) => {
    set(teamExecutionsAtom, (prev) => {
      if (!message.teamId || !message.exploitName) {
        return prev;
      }

      const prevStatus =
        prev[message.exploitName]?.[message.teamId]?.[message.sequence];

      return {
        ...prev,
        [message.teamId]: {
          ...prev[message.teamId],
          [message.exploitName]: {
            ...prev[message.teamId]?.[message.exploitName],
            [message.requestSequence]: {
              published: prevStatus
                ? Math.min(message.published, prevStatus.published)
                : message.published,
              result:
                prevStatus?.result &&
                prevStatus.result.sequence > message.sequence
                  ? prevStatus.result
                  : message,
            },
          },
        },
      };
    });
  },
);

export const exploitExecutionsPurgeDispatch = atom(
  null,
  (_get, set, oldest: number) => {
    set(teamExecutionsAtom, (current) => {
      return Object.fromEntries(
        Object.entries(current).map(([teamId, executionMap]) => [
          teamId,
          Object.fromEntries(
            Object.entries(executionMap).map(([exploit, executions]) => [
              exploit,
              Object.fromEntries(
                Object.entries(executions).filter(
                  ([_, entry]) => entry.published >= oldest,
                ),
              ),
            ]),
          ),
        ]),
      );
    });
  },
);

export const executionStatusAggregateAtom = atom((get) => {
  const teamExecutions = get(teamExecutionsAtom);

  let count = 0;
  let pendingCount = 0;

  // We probably don't want to do FP here to avoid a lot of extra allocations
  for (const [_, exploitMap] of Object.entries(teamExecutions)) {
    for (const [_, executions] of Object.entries(exploitMap)) {
      for (const [_, status] of Object.entries(executions)) {
        ++count;
        if (!status.result) {
          ++pendingCount;
        }
      }
    }
  }

  return {
    count,
    pendingCount,
  };
});

export const teamServiceExecutionAggregateAtom = atom((get) => {
  const teamExecutions = get(teamExecutionsAtom);
  const exploitServiceMap = get(exploitServiceAssociateAtom);

  const pendingCountMap = new Map<string, Map<string, number>>();

  // We probably don't want to do FP here to avoid a lot of extra allocations
  for (const [teamId, exploitMap] of Object.entries(teamExecutions)) {
    const servicePendingCountMap = new Map<string, number>();
    for (const [exploit, executions] of Object.entries(exploitMap)) {
      const service = exploitServiceMap.get(exploit);
      if (!service) continue;

      for (const [_, status] of Object.entries(executions)) {
        if (status.result) {
          continue;
        }
        servicePendingCountMap.set(
          service,
          (servicePendingCountMap.get(service) ?? 0) + 1,
        );
      }
    }
    pendingCountMap.set(teamId, servicePendingCountMap);
  }

  return {
    pendingCountMap,
  };
});
