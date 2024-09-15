import { useInterval } from "usehooks-ts";
import type { TeamServiceFlags } from "../utils/types";
import { Fragment, useState } from "react";
import { FlagCode, flagCodeLookup } from "../utils/enums";

type StatusCellCardProps = {
  flags: TeamServiceFlags;
  teamId: string;
  teamName: string | null;
  serviceName: string;
};

export const StatusCellCard = ({
  flags,
  teamId,
  teamName,
  serviceName,
}: StatusCellCardProps) => {
  const [currentTime, setCurrentTime] = useState(Date.now());

  useInterval(() => {
    setCurrentTime(Date.now());
  }, 1000);

  return (
    <div className="flex flex-col text-sm gap-4">
      <div>
        <div className="font-bold">
          [{teamId}] {teamName}
        </div>
        <div className="text-slate-300">{serviceName}</div>
      </div>
      <div className="grid grid-cols-[auto_1fr_auto] gap-x-4">
        {Object.keys(flags).length === 0 && (
          <span className="text-amber-200">No flags received</span>
        )}
        {Object.entries(flags)
          .reverse()
          .map(([flag, status]) => (
            <Fragment key={flag}>
              <span className="text-slate-300">
                [{flagCodeLookup.get(status.status ?? FlagCode.Unknown)}]
              </span>
              {flag}{" "}
              <span className="text-slate-300">
                {Math.floor((currentTime - status.published) / 1000)}s
              </span>
            </Fragment>
          ))}
      </div>
    </div>
  );
};
