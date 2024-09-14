import { createLazyFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import SimpleDisplay from "../components/SimpleDisplay";
import { flagStatusAggregateAtom } from "../utils/atoms";
import { FlagCode } from "../utils/enums";

function DashboardPage() {
  const flagStatusAggregate = useAtomValue(flagStatusAggregateAtom);

  const statCards = [
    { title: "Executions in queue", value: "?" },
    { title: "Exploits", value: "?" },
    { title: "Flags received", value: flagStatusAggregate.count },
    {
      title: "Accepted flags",
      value:
        (flagStatusAggregate.statusMap.get(FlagCode.Ok) ?? 0) +
        (flagStatusAggregate.statusMap.get(FlagCode.Duplicate) ?? 0),
    },
    {
      title: "Rejected flags",
      value: flagStatusAggregate.statusMap.get(FlagCode.Invalid) ?? 0,
    },
    {
      title: "Pending flags",
      value: flagStatusAggregate.statusMap.get(FlagCode.Pending) ?? 0,
    },
  ];

  return (
    <main className="relative grid grid-rows-[auto_1fr] gap-3 min-h-0 h-[calc(100vh-6rem)]">
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
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
