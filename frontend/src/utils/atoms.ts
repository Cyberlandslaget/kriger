import { atom } from "jotai";
import type { ExploitType } from "./types";

export const configurationAtom = atom({
  flagRegex: "[A-Z0-9]{31}=",
  minutesToFetch: 3 * 60,
});

export const statusAtom = atom({
  start: "2024-01-01T08:00:00.000Z",
  end: "2024-07-04T16:00:00.000Z",
  roundTime: 5,
  rounds: 0,
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
