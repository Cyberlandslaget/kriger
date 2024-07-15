import { atom } from "jotai";

export const statusAtom = atom({
  start: "2024-07-04T09:00:00.000Z",
  end: "2024-07-04T16:00:00.000Z",
  roundTime: 120,
  rounds: 210,
  currentRound: 16,
});
