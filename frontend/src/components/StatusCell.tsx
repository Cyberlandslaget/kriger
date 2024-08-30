import clsx from "clsx";
import { FlagIcon } from "lucide-react";
import { useMemo } from "react";
import { FLAG_CODE, flagCodeLookup } from "../utils/enums";
import type { TeamServiceFlags } from "../utils/types";

type StatusCellProps = {
  flags: TeamServiceFlags;
};

export const StatusCell = ({ flags }: StatusCellProps) => {
  const aggregate = useMemo(() => {
    return Object.values(flags).reduce((map, { status }) => {
      const key = status ?? FLAG_CODE.Pending;
      map.set(key, (map.get(key) ?? 0) + 1);
      return map;
    }, new Map<FLAG_CODE, number>());
  }, [flags]);

  const borderColor = useMemo(() => {
    if (
      aggregate.get(FLAG_CODE.Error) ||
      aggregate.get(FLAG_CODE.Unknown) ||
      aggregate.get(FLAG_CODE.Invalid) ||
      aggregate.get(FLAG_CODE.Own) ||
      aggregate.get(FLAG_CODE.Nop)
    ) {
      return "border-red-500/50";
    }
    if (aggregate.get(FLAG_CODE.Resubmit) || aggregate.get(FLAG_CODE.Old)) {
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
        "bg-slate-950 bg-opacity-20 border-2 rounded-sm",
        borderColor,
      )}
      title={aggregateSummary}
    >
      <div className="flex flex-row items-center p-1.5 gap-1">
        {[
          ...Array(
            (aggregate.get(FLAG_CODE.Ok) ?? 0) +
              (aggregate.get(FLAG_CODE.Duplicate) ?? 0),
          ),
        ].map((_, i) => (
          <FlagIcon
            className="stroke-green-500 fill-green-500 w-5 h-5"
            key={i}
          />
        ))}
        {/* TODO: We probably want to limit the amount of pending flags here since they may overflow. */}
        {[...Array(aggregate.get(FLAG_CODE.Pending) ?? 0)].map((_, i) => (
          <FlagIcon
            className="stroke-slate-700 fill-slate-700 w-5 h-5"
            key={i}
          />
        ))}
      </div>
    </td>
  );
};
