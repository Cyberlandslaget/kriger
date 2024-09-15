import AutoSizer from "react-virtualized-auto-sizer";
import { forwardRef } from "react";
import { useAtomValue } from "jotai";
import {
  servicesAtom,
  teamFlagStatusAtom,
  teamsAtom,
  teamServiceExecutionAggregateAtom,
} from "../utils/atoms";
import { StatusCell } from "./StatusCell";
import { FixedSizeList as List } from "react-window";

function SimpleDisplay() {
  const services = useAtomValue(servicesAtom);
  const teams = useAtomValue(teamsAtom);
  const teamFlagMap = useAtomValue(teamFlagStatusAtom);
  const teamServiceExecutionAggregate = useAtomValue(
    teamServiceExecutionAggregateAtom,
  );

  return (
    <div className="flex flex-col h-full relative rounded-md">
      <AutoSizer>
        {({ height, width }) => (
          <List
            height={height}
            itemCount={Object.keys(teams).length}
            itemSize={48}
            width={width}
            innerElementType={forwardRef(({ children, ...rest }, ref) => (
              <table ref={ref} {...rest} className="relative">
                <thead className="sticky top-0 bg-primary-bg h-10 z-10">
                  <tr className="flex mb-2">
                    <th className="min-w-48 max-w-72 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-left">
                      Team
                    </th>
                    {services.map((service) => (
                      <th
                        className="w-full min-w-36 max-w-72 ml-2 h-10 font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-nowrap truncate"
                        key={service.name}
                        title={service.name}
                      >
                        {service.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>{children}</tbody>
              </table>
            ))}
          >
            {({ index, style }) => {
              const [teamId, team] = Object.entries(teams)[index];
              return (
                <tr
                  key={`key-${index}`}
                  style={{ ...style }}
                  className="flex min-w-full mt-12"
                >
                  <td
                    className="min-w-48 max-w-72 h-10 bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2"
                    title={`[${teamId}] ${team.name ?? ""}`}
                  >
                    <div className="w-full flex items-center text-sm p-1.5 h-full shadow-inner  rounded-sm transition-all duration-150 truncate">
                      [{teamId}] {team.name}
                    </div>
                  </td>

                  {services.map((service) => (
                    <StatusCell
                      flags={teamFlagMap[teamId]?.[service.name] ?? {}}
                      hasPendingExecution={
                        (teamServiceExecutionAggregate.pendingCountMap
                          ?.get(teamId)
                          ?.get(service.name) ?? 0) > 0
                      }
                      teamId={teamId}
                      teamName={team.name}
                      serviceName={service.name}
                      key={service.name}
                    />
                  ))}
                </tr>
              );
            }}
          </List>
        )}
      </AutoSizer>
    </div>
  );
}

export default SimpleDisplay;
