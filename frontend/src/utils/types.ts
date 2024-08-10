import type { FLAG_CODE } from "./enums";

export type TeamFlagMap = {
  [teamId: string]: TeamServiceMap;
};

export type TeamServiceMap = {
  [service: string]: TeamServiceFlags;
};

export type TeamServiceFlags = {
  [flag: string]: TeamFlagStatus;
};

export type TeamFlagStatus = {
  status?: FLAG_CODE;
  published: number;
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
