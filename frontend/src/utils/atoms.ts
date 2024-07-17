import { atom } from "jotai";
import type { ExploitType } from "./types";
import type { ExtendedType } from "./enums";

export const configurationAtom = atom({
  flagRegex: "[A-Z0-9]{31}=",
  minutesToFetch: 3 * 60,
});

export const statusAtom = atom({
  start: "2024-07-04T09:00:00.000Z",
  end: "2024-07-04T16:00:00.000Z",
  roundTime: 120,
  rounds: 210,
  currentRound: 16,
});
export const currentTickAtom = atom((get) => get(statusAtom).currentRound);

export const exploitsAtom = atom<ExploitType[] | null>(null);
export const extendedSelectionAtom = atom<{
  type: ExtendedType | null;
  selection: string | null;
}>({ type: null, selection: null });
