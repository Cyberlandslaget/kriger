export enum FlagCode {
  // Internal status code used by the frontend to represent "pending" flags
  Pending = -1,

  Ok = 1, // ACCEPTED: flag claimed
  Duplicate = 2, // DENIED: flag already claimed
  Own = 3, // DENIED: flag is your own
  Nop = 4, // DENIED: flag from nop team
  Old = 5, // DENIED: flag too old
  Invalid = 6, // DENIED: invalid flag

  /// The server explicitly requests the flag to be resubmitted.
  /// This can be due to the fact that the flag is not yet valid.
  /// Submitters should retry this status.
  Resubmit = 7, // RESUBMIT: the flag is not active yet, wait for next round

  /// Server refused flag. Pre- or post-competition.
  /// Submitters should retry this status.
  Error = 8, // ERROR: notify the organizers and retry later

  /// Unknown response. Submitters should definitely retry this status.
  Unknown = 200,
}
export enum ServiceStatus {
  OK = 0,
  DOWN = 1,
  SYSTEM_ERROR = -1,
}

export const flagCodeLookup = new Map<FlagCode | string, string>(
  Object.entries(FlagCode).map(([k, v]) => [v, k]),
);
