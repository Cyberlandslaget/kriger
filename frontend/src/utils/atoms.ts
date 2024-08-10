import { atom } from "jotai";
import type {
  FlagSubmissionMessage,
  FlagSubmissionResultMessage,
  Service,
  Team,
} from "../services/models";
import type { ExploitType, TeamFlagMap } from "./types";
import { FLAG_CODE } from "./enums";

export const competitionConfigAtom = atom({
  start: "1990-01-01T08:00:00.000Z",
  tick: 120,
  flagFormat: "",
  flagValidity: 5,
});

export const statusAtom = atom({
  currentTick: -1,
});

export const currentTickAtom = atom(
  (get) => get(statusAtom).currentTick,
  (get, set, currentTick: number) =>
    set(statusAtom, {
      ...get(statusAtom),
      currentTick,
    }),
);

export const exploitsAtom = atom<ExploitType[] | null>(null);
export const executionsAtom = atom<ExploitType[] | null>(null);

export const teamsAtom = atom<Record<string, Team>>({});
export const servicesAtom = atom<Service[]>([]);

export const teamFlagStatusAtom = atom<TeamFlagMap>({});

export const teamFlagSubmissionDispatch = atom(
  null,
  (get, set, message: FlagSubmissionMessage | FlagSubmissionResultMessage) => {
    const prev = get(teamFlagStatusAtom);

    if (!message.teamId || !message.service) {
      return;
    }

    set(teamFlagStatusAtom, {
      ...prev,
      [message.teamId]: {
        ...prev[message.teamId],
        [message.service]: {
          ...prev[message.teamId]?.[message.service],
          [message.flag]: {
            status: "status" in message ? message.status : undefined,
            published:
              "status" in message
                ? (prev[message.teamId]?.[message.service]?.[message.flag]
                    ?.published ?? message.service)
                : message.published,
          },
        },
      },
    });
  },
);

export const teamFlagPurgeDispatch = atom(null, (get, set, oldest: number) => {
  set(
    teamFlagStatusAtom,
    Object.fromEntries(
      Object.entries(get(teamFlagStatusAtom)).map(
        ([teamId, teamServiceMap]) => [
          teamId,
          Object.fromEntries(
            Object.entries(teamServiceMap).map(([service, flags]) => [
              service,
              Object.fromEntries(
                Object.entries(flags).filter(
                  ([_, entry]) => entry.published > oldest,
                ),
              ),
            ]),
          ),
        ],
      ),
    ),
  );
});

// TODO: Add tiered caching? We are aggregating everything every time 'teamFlagStatusAtom' updates.
// Premature optimization here can lead to inconsistent aggregation. We have to deeal with status updates,
// purging and the message delivery order.
export const flagStatusAggregateAtom = atom((get) => {
  const flagStatus = get(teamFlagStatusAtom);

  let count = 0;
  const map = new Map<FLAG_CODE, number>();

  // We probably don't want to do FP here to avoid a lot of extra allocations
  for (const [_, serviceMap] of Object.entries(flagStatus)) {
    for (const [_, serviceFlags] of Object.entries(serviceMap)) {
      for (const [_, status] of Object.entries(serviceFlags)) {
        const key = status.status ?? FLAG_CODE.Pending;
        map.set(key, (map.get(key) ?? 0) + 1);
        ++count;
      }
    }
  }

  return {
    count,
    statusMap: map,
  };
});
