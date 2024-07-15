import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/config")({
  component: () => <div>Config!</div>,
});

