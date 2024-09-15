import clsx from "clsx";
import { FlagIcon, LoaderCircleIcon } from "lucide-react";
import { useMemo } from "react";
import { FlagCode } from "../utils/enums";
import type { TeamServiceFlags } from "../utils/types";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./HoverCard";
import { StatusCellCard } from "./StatusCellCard";

type StatusCellProps = {
  flags: TeamServiceFlags;
  hasPendingExecution: boolean;
  teamId: string;
  teamName: string | null;
  serviceName: string;
};

export const StatusCell = ({
  flags,
  teamId,
  teamName,
  serviceName,
  hasPendingExecution,
}: StatusCellProps) => {
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
    if (
      aggregate.get(FlagCode.Resubmit) ||
      aggregate.get(FlagCode.Old) ||
      aggregate.get(FlagCode.Stale)
    ) {
      return "border-amber-200/40";
    }

    return "border-slate-950/20";
  }, [aggregate]);

  return (
    <td
      className={clsx(
        "w-full min-w-36 max-w-72 ml-2 h-10 bg-slate-950 bg-opacity-20 border-2 rounded-sm",
        borderColor,
      )}
    >
      <HoverCard openDelay={50} closeDelay={50}>
        <HoverCardTrigger>
          <div className="flex flex-row items-center p-1.5 gap-1 h-full">
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
            <div className="flex-1" />
            {hasPendingExecution && (
              <LoaderCircleIcon className="animate-spin stroke-slate-400" />
            )}
          </div>
        </HoverCardTrigger>
        <HoverCardContent align="start">
          <StatusCellCard
            flags={flags}
            serviceName={serviceName}
            teamId={teamId}
            teamName={teamName}
          />
        </HoverCardContent>
      </HoverCard>
    </td>
  );
};
