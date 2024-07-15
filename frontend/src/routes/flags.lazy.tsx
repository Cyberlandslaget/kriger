import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/flags")({
  component: () => <div>Flags!</div>,
});

