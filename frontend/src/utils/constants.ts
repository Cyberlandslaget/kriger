export const CONFIG = {
  webSocketUrl: new URL(
    import.meta.env.VITE_WS_URL ?? "http://localhost:8001",
    location.origin,
  ),
  restUrl: new URL(
    import.meta.env.VITE_REST_URL ?? "http://localhost:8000",
    location.origin,
  ),
};

