import { createLazyFileRoute } from "@tanstack/react-router";
import ExecutionDisplay from "../components/ExecutionDisplay";

function Executions() {
  return (
    <main className="relative grid grid-rows-[30px_1fr] gap-3 min-h-0 h-[calc(100vh-6rem)]">
      <ExecutionDisplay />
    </main>
  );
}

export const Route = createLazyFileRoute("/executions")({
  component: () => Executions(),
});
