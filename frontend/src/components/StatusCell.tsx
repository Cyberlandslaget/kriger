import clsx from "clsx";
import { FlagIcon } from "lucide-react";
import { useMemo } from "react";
import { FlagCode, flagCodeLookup } from "../utils/enums";
import type { TeamServiceFlags } from "../utils/types";

type StatusCellProps = {
  flags: TeamServiceFlags;
};

export const StatusCell = ({ flags }: StatusCellProps) => {
  const aggregate = useMemo(() => {
    return Object.values(flags).reduce((map, { status }) => {
      const key = status ?? FlagCode.Pending;
      map.set(key, (map.get(key) ?? 0) + 1);
      return map;
    }, new Map<FlagCode, number>());
  }, [flags]);

  const borderColor = useMemo(() => {
    if (
      aggregate.get(FlagCode.Error) ||
      aggregate.get(FlagCode.Unknown) ||
      aggregate.get(FlagCode.Invalid) ||
      aggregate.get(FlagCode.Own) ||
      aggregate.get(FlagCode.Nop)
    ) {
      return "border-red-500/50";
    }
    if (aggregate.get(FlagCode.Resubmit) || aggregate.get(FlagCode.Old)) {
      return "border-amber-200/40";
    }

    return "border-slate-950/20";
  }, [aggregate]);

  const aggregateSummary = useMemo(() => {
    return Array.from(aggregate)
      .map(([code, count]) => `${flagCodeLookup.get(code)}: ${count}`)
      .join("\n");
  }, [aggregate]);

  return (
    <td
      className={clsx(
        "w-full min-w-36 max-w-72 ml-2 h-10 bg-slate-950 bg-opacity-20 border-2 rounded-sm",
        borderColor,
      )}
      title={aggregateSummary}
    >
      <div className="flex flex-row items-center p-1.5 gap-1">
        {[
          ...Array(
            (aggregate.get(FlagCode.Ok) ?? 0) +
              (aggregate.get(FlagCode.Duplicate) ?? 0),
          ),
        ].map((_, i) => (
          <FlagIcon
            className="stroke-green-500 fill-green-500 w-5 h-5"
            // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
            key={i}
          />
        ))}
        {/* TODO: We probably want to limit the amount of pending flags here since they may overflow. */}
        {[...Array(aggregate.get(FlagCode.Pending) ?? 0)].map((_, i) => (
          <FlagIcon
            className="stroke-slate-700 fill-slate-700 w-5 h-5"
            // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
            key={i}
          />
        ))}
      </div>
    </td>
  );
};
