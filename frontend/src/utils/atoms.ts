import { atom } from "jotai";
import type { ExploitType } from "./types";

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
