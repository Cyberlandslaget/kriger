import { atom } from "jotai";
import type {
  FlagSubmissionMessage,
  FlagSubmissionResultMessage,
  Service,
  Team,
} from "../services/models";
import type { ExploitType, TeamFlagMap } from "./types";

export const competitionConfigAtom = atom({
  start: "1990-01-01T08:00:00.000Z",
  tick: 120,
  flagFormat: "",
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
          },
        },
      },
    });
  },
);
