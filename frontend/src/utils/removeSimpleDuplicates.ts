import { FLAG_CODE } from "./enums";
import type { FlagType } from "./types";

type Ticks = {
  [key: number]: FlagType | null;
};
type ServiceTicks = {
  [key: string]: Ticks;
};
type Challenges = {
  [key: string]: ServiceTicks;
};

export const removeSimpleDuplicates = (
  teams: string[],
  services: string[],
  flags: FlagType[],
  currentTick: number,
  exploitId?: number,
) => {
  const newFlags: Challenges = {};

  // Generate ticks by team services
  for (const teamId of teams) {
    const serviceTicks: ServiceTicks = {};
    for (const service of services) {
      const ticks: Ticks = {};
      for (let tick = currentTick; tick > 0; tick--) {
        ticks[tick] = null;
      }
      serviceTicks[service] = ticks;
    }
    newFlags[teamId] = serviceTicks;
  }

  // Get total amount of services from current tick
  for (const oldFlag of flags) {
    try {
      const newFlag =
        newFlags[oldFlag?.team]?.[oldFlag?.service]?.[oldFlag?.target_tick];
      if (exploitId && oldFlag?.exploit_id !== exploitId) continue;
      if (!newFlag || FLAG_CODE[oldFlag.status] > FLAG_CODE[newFlag.status])
        newFlags[oldFlag.team][oldFlag.service][oldFlag.target_tick] = oldFlag;
    } catch (_e) {}
  }
  return newFlags;
};
