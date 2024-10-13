// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { mapWebSocketMessage } from "./models";
import type { WebSocketMessage } from "./models";

export class WebSocketService {
  readonly #url: string;
  readonly #fromProvider: () => number;
  readonly #messageHandler: (message: WebSocketMessage) => void;

  #ws: WebSocket | undefined;
  #closed = false;
  #timer: number | undefined;

  constructor(
    url: string,
    fromProvider: () => number,
    messageHandler: (message: WebSocketMessage) => void,
  ) {
    this.#url = url;
    this.#fromProvider = fromProvider;
    this.#messageHandler = messageHandler;
    this.connect();
  }

  connect() {
    // Sanity check
    if (this.#closed) return;

    // Close any existing connections
    this.#ws?.close();

    console.info("[ws] connecting");

    const url = new URL(this.#url);
    url.searchParams.append("from", this.#fromProvider().toString());

    this.#ws = new WebSocket(url);
    this.#ws.onopen = () => {
      console.info("[ws] connected");
    };
    this.#ws.onclose = () => {
      console.info("[ws] disconnected");

      // Don't attempt to reconnect if `close()` has been explicitly called
      if (this.#closed) return;

      // Apply jitter to avoid a thundering herd
      const delay = 1000 + Math.floor(Math.random() * 100);
      this.#timer = setTimeout(() => {
        this.connect();
      }, delay);
    };
    this.#ws.onmessage = (event) => {
      let message: WebSocketMessage;
      try {
        message = mapWebSocketMessage(JSON.parse(event.data));
      } catch (error) {
        console.warn("[ws] malformed data received", event.data, error);
        return;
      }
      try {
        this.#messageHandler(message);
      } catch (error) {
        console.warn("[ws] unable to handle message", message, error);
      }
    };
  }

  close() {
    this.#closed = true;
    this.#ws?.close();
    this.#ws = undefined;

    if (this.#timer) {
      clearTimeout(this.#timer);
    }
  }
}
