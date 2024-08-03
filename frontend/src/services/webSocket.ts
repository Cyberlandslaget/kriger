import { mapWebSocketMessage } from "./models";
import type { WebSocketMessage } from "./models";

export class WebSocketService {
  #url: string;
  #ws: WebSocket | undefined;
  #closed = false;
  #timer: number | undefined;
  #messageHandler: (message: WebSocketMessage) => void;

  constructor(
    url: string,
    messageHandler: (message: WebSocketMessage) => void,
  ) {
    this.#url = url;
    this.#messageHandler = messageHandler;
    this.connect();
  }

  connect() {
    // Sanity check
    if (this.#closed) return;

    // Close any existing connections
    this.#ws?.close();

    console.info("[ws] connecting");

    this.#ws = new WebSocket(this.#url);
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
      try {
        const message = mapWebSocketMessage(JSON.parse(event.data));
        this.#messageHandler(message);
      } catch (error) {
        console.warn("[ws] malformed data received", event.data);
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
