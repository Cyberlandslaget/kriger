import { createLazyFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import SimpleDisplay from "../components/SimpleDisplay";
import { flagStatusAggregateAtom } from "../utils/atoms";
import { FLAG_CODE } from "../utils/enums";

function DashboardPage() {
  const flagStatusAggregate = useAtomValue(flagStatusAggregateAtom);

  const statCards = [
    { title: "Executions in queue", value: "26" },
    { title: "Exploits", value: "3" },
    { title: "Flags received", value: flagStatusAggregate.count },
    {
      title: "Accepted flags",
      value: flagStatusAggregate.statusMap.get(FLAG_CODE.Ok) ?? 0,
    },
    {
      title: "Rejected flags",
      value: flagStatusAggregate.statusMap.get(FLAG_CODE.Invalid) ?? 0,
    },
    {
      title: "Pending flags",
      value: flagStatusAggregate.statusMap.get(FLAG_CODE.Pending) ?? 0,
    },
  ];

  return (
    <main className="flex flex-col gap-3">
      <div className="grid grid-cols-6 gap-3">
        {statCards.map((box) => (
          <div
            key={box.title}
            className="px-3 py-2 shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm flex flex-col"
          >
            <span className="text-xs">{box.title}</span>
            <span className="text-sm font-bold">{box.value}</span>
          </div>
        ))}
      </div>
      <SimpleDisplay />
    </main>
  );
}

export const Route = createLazyFileRoute("/")({
  component: DashboardPage,
});
