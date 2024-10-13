// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { useAtomValue } from "jotai";
import {
  executionStatusAggregateAtom,
  exploitsAtom,
  flagStatusAggregateAtom,
} from "../utils/atoms";
import { FlagCode } from "../utils/enums";

export const StatsCards = () => {
  const flagStatusAggregate = useAtomValue(flagStatusAggregateAtom);
  const executionStatusAggregate = useAtomValue(executionStatusAggregateAtom);
  const exploits = useAtomValue(exploitsAtom);

  const statCards = [
    {
      title: "Pending executions",
      value: executionStatusAggregate.pendingCount,
    },
    { title: "Exploits", value: exploits?.length },
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
  );
};
