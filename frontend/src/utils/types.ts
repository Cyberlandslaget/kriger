// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import type { ExecutionResultMessage } from "../services/models";
import type { FlagCode } from "./enums";

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
  status?: FlagCode;
  published: number;
  exploit: string | null;
};

export type TeamExecutionMap = {
  [teamId: string]: ExploitExecutionMap;
};

export type ExploitExecutionMap = {
  [exploit: string]: TeamExploitExecutions;
};

export type TeamExploitExecutions = {
  [sequence: number]: TeamExecutionStatus;
};

export type TeamExecutionStatus = {
  published: number;
  result?: ExecutionResultMessage;
};
