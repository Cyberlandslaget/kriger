export enum FLAG_CODE {
  OK = "OK", // Accepted: X flag points
  INV = "INV", // Denied: invalid flag
  NOP = "NOP", // Denied: flag from nop team
  OWN = "OWN", // Denied: flag is your own
  OLD = "OLD", // Denied: flag too old
  DUP = "DUP", // Denied: flag already claimed
  ERR = "ERR", // Error: <<ERROR>>
}
export enum SERVICE_STATUS {
  OK = 0,
  DOWN = 1,
  SYSTEM_ERROR = -1,
}
export enum ExtendedType {
  Team = "team",
  Service = "service",
}
