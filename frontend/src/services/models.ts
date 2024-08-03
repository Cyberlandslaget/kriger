import type { FLAG_CODE } from "../utils/enums";

type RawWebSocketMessageTemplate<
  TName extends string,
  TPayload extends object,
> = {
  /**
   * The name of the message
   */
  t: TName;
  /**
   * The payload of the message
   */
  d: TPayload;
};
type RawSchedulingStartMessage = RawWebSocketMessageTemplate<
  "scheduling_start",
  {
    /**
     * The current tick
     */
    i: number;
  }
>;
type RawFlagSubmissionMessage = RawWebSocketMessageTemplate<
  "flag_submission",
  {
    /**
     * The flag
     */
    f: string;
    /**
     * The team id
     */
    t: string | null;
    /**
     * The service name
     */
    s: string | null;
    /**
     * The exploit name
     */
    e: string | null;
  }
>;
type RawFlagSubmissionResultMessage = RawWebSocketMessageTemplate<
  "flag_submission_result",
  {
    /**
     * The flag
     */
    f: string;
    /**
     * The team id
     */
    t: string | null;
    /**
     * The service name
     */
    s: string | null;
    /**
     * The exploit name
     */
    e: string | null;
    /**
     * The flag submission result status
     */
    r: FLAG_CODE;
    /**
     * The points received
     */
    p: number | null;
  }
>;
type RawWebSocketMessage =
  | RawSchedulingStartMessage
  | RawFlagSubmissionMessage
  | RawFlagSubmissionResultMessage;

export type SchedulingStartMessage = {
  type: "scheduling_start";
  tick: number;
};

export type FlagSubmissionMessage = {
  type: "flag_submission";
  flag: string;
  teamId: string | null;
  service: string | null;
  exploit: string | null;
};

export type FlagSubmissionResultMessage = {
  type: "flag_submission_result";
  flag: string;
  teamId: string | null;
  service: string | null;
  exploit: string | null;
  status: FLAG_CODE;
  points: number | null;
};

export type WebSocketMessage =
  | SchedulingStartMessage
  | FlagSubmissionMessage
  | FlagSubmissionResultMessage;

export class SchedulingStartMessageWrapper implements SchedulingStartMessage {
  #raw: RawSchedulingStartMessage;

  constructor(raw: RawSchedulingStartMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get tick() {
    return this.#raw.d.i;
  }
}
export class FlagSubmissionMessageWrapper implements FlagSubmissionMessage {
  #raw: RawFlagSubmissionMessage;

  constructor(raw: RawFlagSubmissionMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get flag() {
    return this.#raw.d.f;
  }

  get teamId() {
    return this.#raw.d.t;
  }

  get service() {
    return this.#raw.d.s;
  }

  get exploit() {
    return this.#raw.d.e;
  }
}

export class FlagSubmissionResultMessageWrapper
  implements FlagSubmissionResultMessage
{
  #raw: RawFlagSubmissionResultMessage;

  constructor(raw: RawFlagSubmissionResultMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get flag() {
    return this.#raw.d.f;
  }

  get teamId() {
    return this.#raw.d.t;
  }

  get service() {
    return this.#raw.d.s;
  }

  get exploit() {
    return this.#raw.d.e;
  }

  get status() {
    return this.#raw.d.r;
  }

  get points() {
    return this.#raw.d.p;
  }
}

export const mapWebSocketMessage = (
  raw: RawWebSocketMessage,
): WebSocketMessage => {
  switch (raw.t) {
    case "scheduling_start":
      return new SchedulingStartMessageWrapper(raw);
    case "flag_submission":
      return new FlagSubmissionMessageWrapper(raw);
    case "flag_submission_result":
      return new FlagSubmissionResultMessageWrapper(raw);
    default:
      throw new Error(`Unsupported message: ${JSON.stringify(raw)}`);
  }
};
