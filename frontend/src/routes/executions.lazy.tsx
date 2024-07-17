import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/executions")({
  component: () => (
    <main className="flex flex-col gap-6">
      <div>
        <input
          className="w-full bg-slate-950/80 px-2 py-1 text-slate-300 rounded-sm"
          type="text"
          value={""}
        />
      </div>
    </main>
  ),
});
