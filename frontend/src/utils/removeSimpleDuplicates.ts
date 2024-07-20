import { FLAG_CODE } from "./enums";
import type { FlagSubmissionResult } from "./types";

type Ticks = {
  [key: number]: FlagSubmissionResult | null;
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
  flags: FlagSubmissionResult[],
  currentTick: number,
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
    if (!oldFlag?.team_id || !oldFlag.service) continue;
    try {
      const newFlag =
        newFlags[oldFlag?.team_id]?.[oldFlag?.service]?.[oldFlag?.tick];
      if (!newFlag || oldFlag.status > newFlag.status)
        newFlags[oldFlag.team_id][oldFlag.service][oldFlag.tick] = oldFlag;
    } catch (_e) {}
  }
  return newFlags;
};
