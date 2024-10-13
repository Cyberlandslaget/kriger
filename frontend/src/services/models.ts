// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import type { ExecutionResultStatusCode, FlagCode } from "../utils/enums";

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
  /**
   * The publishing time of the message in Unix millis timestamp (UTC)
   */
  p: number;
  /**
   * The stream sequence number of the message
   */
  s: number;
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
    r: FlagCode;
    /**
     * The points received
     */
    p: number | null;
  }
>;

type RawExecutionRequestMessage = RawWebSocketMessageTemplate<
  "execution_request",
  {
    /**
     * The exploit name
     */
    n?: string;

    /**
     * The target IP address
     */
    a: string;

    /**
     * The flag hint
     */
    h?: unknown;

    /**
     * The team id
     */
    t?: string;
  }
>;

type RawExecutionResultMessage = RawWebSocketMessageTemplate<
  "execution_result",
  {
    /**
     * The exploit name
     */
    n?: string;

    /**
     * The team id
     */
    t?: string;

    /**
     * The time taken to execute the exploit in milliseconds
     */
    d: number;

    /**
     * The execution process' exit code
     */
    e?: number;

    /**
     * The execution's resulting status
     */
    s: ExecutionResultStatusCode;

    /**
     * The request sequence
     */
    r: number;

    /**
     * The execution attempt number
     */
    a?: number;
  }
>;

type RawWebSocketMessage =
  | RawSchedulingStartMessage
  | RawFlagSubmissionMessage
  | RawFlagSubmissionResultMessage
  | RawExecutionRequestMessage
  | RawExecutionResultMessage;

export type SchedulingStartMessage = {
  type: "scheduling_start";
  published: number;
  tick: number;
};

export type FlagSubmissionMessage = {
  type: "flag_submission";
  published: number;
  flag: string;
  teamId: string | null;
  service: string | null;
  exploit: string | null;
};

export type FlagSubmissionResultMessage = {
  type: "flag_submission_result";
  published: number;
  flag: string;
  teamId: string | null;
  service: string | null;
  exploit: string | null;
  status: FlagCode;
  points: number | null;
};

export type ExecutionRequestMessage = {
  type: "execution_request";
  published: number;
  sequence: number;
  exploitName?: string;
  ipAddress: string;
  flagHint?: unknown;
  teamId?: string;
};

export type ExecutionResultMessage = {
  type: "execution_result";
  published: number;
  sequence: number;
  exploitName?: string;
  teamId?: string;
  time: number;
  exitCode?: number;
  status: ExecutionResultStatusCode;
  requestSequence: number;
  attempt?: number;
};

export type WebSocketMessage =
  | SchedulingStartMessage
  | FlagSubmissionMessage
  | FlagSubmissionResultMessage
  | ExecutionRequestMessage
  | ExecutionResultMessage;

export class SchedulingStartMessageWrapper implements SchedulingStartMessage {
  #raw: RawSchedulingStartMessage;

  constructor(raw: RawSchedulingStartMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get published() {
    return this.#raw.p;
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

  get published() {
    return this.#raw.p;
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

  get published() {
    return this.#raw.p;
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

export class ExecutionRequestMessageWrapper implements ExecutionRequestMessage {
  #raw: RawExecutionRequestMessage;

  constructor(raw: RawExecutionRequestMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get published() {
    return this.#raw.p;
  }

  get sequence() {
    return this.#raw.s;
  }

  get exploitName() {
    return this.#raw.d.n;
  }

  get ipAddress() {
    return this.#raw.d.a;
  }

  get flagHint() {
    return this.#raw.d.h;
  }

  get teamId() {
    return this.#raw.d.t;
  }
}

export class ExecutionResultMessageWrapper implements ExecutionResultMessage {
  #raw: RawExecutionResultMessage;

  constructor(raw: RawExecutionResultMessage) {
    this.#raw = raw;
  }

  get type() {
    return this.#raw.t;
  }

  get published() {
    return this.#raw.p;
  }

  get sequence() {
    return this.#raw.s;
  }

  get exploitName() {
    return this.#raw.d.n;
  }

  get teamId() {
    return this.#raw.d.t;
  }

  get time() {
    return this.#raw.d.d;
  }

  get status() {
    return this.#raw.d.s;
  }

  get requestSequence() {
    return this.#raw.d.r;
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
    case "execution_request":
      return new ExecutionRequestMessageWrapper(raw);
    case "execution_result":
      return new ExecutionResultMessageWrapper(raw);
    default:
      throw new Error(`Unsupported message: ${JSON.stringify(raw)}`);
  }
};

export type APISuccessResponse<T> = {
  data: T;
};

export type APIErrorResponse = {
  error: {
    message: string;
  };
};

export type APIResponse<T> = APISuccessResponse<T> | APIErrorResponse;

export type ServerConfig = {
  competition: CompetitionConfig;
};

export type CompetitionConfig = {
  start: string;
  tick: number;
  tickStart: number;
  flagValidity: number;
  flagFormat: string;
};

export type Service = {
  name: string;
  hasHint: boolean;
};

export type Team = {
  name: string | null;
  ipAddress: string | null;
  services: Record<string, string>;
};

export type Exploit = {
  manifest: ExploitManifest;
  image: string;
};

export type ExploitManifest = {
  name: string;
  service: string;
  replicas: number;
  workers: number | null;
  enabled: boolean;
  resources: ExploitResources;
};

export type ExploitResources = {
  cpuRequest: string | null;
  memRequest: string | null;
  cpuLimit: string;
  memLimit: string;
  timeout: number;
};

export type ExecutionMap = {
  executions: { [sequence: number]: Execution };
  sortedSequence: number[];
};
export type Execution = {
  type: string;
  published: number;
  sequence: number;
  exploitName: string;
  ipAddress: string;
  flagHint: string;
  teamId: string;
  time?: number;
  exitCode?: number;
  status?: ExecutionResultStatusCode;
  attempt?: number;
  requestSequence?: number;
};
