// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

export enum FlagCode {
  // Internal status code used by the frontend to represent "pending" flags
  Pending = -1,

  Ok = 1, // ACCEPTED: flag claimed
  Duplicate = 2, // DENIED: flag already claimed
  Own = 3, // DENIED: flag is your own
  Nop = 4, // DENIED: flag from nop team
  Old = 5, // DENIED: flag too old
  Invalid = 6, // DENIED: invalid flag

  /**
   * The server explicitly requests the flag to be resubmitted.
   * This can be due to the fact that the flag is not yet valid.
   * Submitters should retry this status.
   */
  Resubmit = 7,

  /**
   * Server refused flag. Pre- or post-competition.
   * Submitters should retry this status.
   */
  Error = 8,

  /**
   * The flag that was placed by the checker is stale and is invalid.
   * Submitters should not retry this status.
   */
  Stale = 9,

  /**
   * Unknown response. Submitters should definitely retry this status.
   */
  Unknown = 200,
}
export enum ServiceStatus {
  OK = 0,
  DOWN = 1,
  SYSTEM_ERROR = -1,
}
export enum ExecutionResultStatusCode {
  Success = 0,
  Timeout = 1,
  Error = 2,
}

export const flagCodeLookup = new Map<FlagCode | string, string>(
  Object.entries(FlagCode).map(([k, v]) => [v, k]),
);
