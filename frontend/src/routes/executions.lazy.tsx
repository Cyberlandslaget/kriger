import { createLazyFileRoute } from "@tanstack/react-router";
import { useCallback } from "react";

export const Route = createLazyFileRoute("/executions")({
  component: () => executions(),
});

function executions() {
  const onChangeHandler = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      // TODO
    },
    [],
  );

  return (
    <main className="flex flex-col gap-6">
      <div>
        <input
          className="w-full bg-slate-950/80 px-2 py-1 text-slate-300 rounded-sm"
          type="text"
          value={""}
          onChange={onChangeHandler}
        />
      </div>
    </main>
  );
}
