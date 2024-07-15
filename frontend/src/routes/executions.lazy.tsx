import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/executions")({
  component: () => <div>Executions!</div>,
});

