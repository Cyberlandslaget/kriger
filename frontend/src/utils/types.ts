import type { FLAG_CODE } from "./enums";

export type ScoreboardType = {
  teams: {
    [key: string]: {
      ip_address: string;
      name?: string;
      services: {
        [key: string]: number;
      };
    };
  };
};

export type ExecutionType = {
  id: number;
  exploit_id: number;
  output: string;
  exit_code: number;
  started_at: string;
  finished_at: string;
  target_id: number;

  service: string;
  target_tick: number;
  team: string;
};

export type ExploitType = {
  id: number;
  name: string;
  enabled: boolean;
  service: string;
  pool_size: number;
  docker_image: string;
  docker_containers: string[];
  blacklist: string[];
};

export type FlagSubmissionResult = {
  flag: string; // f
  team_id?: string; // t
  service?: string; // s
  exploit?: string; // e
  status: FLAG_CODE; // r
  points?: number; // p
  tick: number; // c
};
