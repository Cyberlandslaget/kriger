import { useAtomValue } from "jotai";
import { servicesAtom, teamFlagStatusAtom, teamsAtom } from "../utils/atoms";
import { StatusCell } from "./StatusCell";

function SimpleDisplay() {
  const services = useAtomValue(servicesAtom);
  const teams = useAtomValue(teamsAtom);
  const teamFlagMap = useAtomValue(teamFlagStatusAtom);

  return (
    <div className="relative wrapper my-1 gap-2 h-[calc(100%-1.5rem)]">
      <div className="w-full h-full rounded-md overflow-auto">
        <div className="grid [grid-template-columns:13rem_1fr] gap-2 min-h-0">
          <div className="flex flex-col gap-1">
            <p className="flex mb-1 items-center font-bold text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden">
              Team
            </p>
            {Object.entries(teams).map(([teamId, team]) => (
              <div
                key={teamId}
                className="flex items-center text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm transition-all duration-150"
                title={`[${teamId}] ${team.name ?? ""}`}
              >
                <p className="truncate">
                  [{teamId}] {team.name}
                </p>
              </div>
            ))}
          </div>

          <div className="flex gap-1 overflow-auto w-full pb-1">
            {services.map((service) => (
              <div key={service.name} className="flex flex-col gap-1">
                <p className="flex w-44 mb-1 items-center justify-center font-bold text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden transition-all duration-150">
                  {service.name}
                </p>
                {Object.keys(teams).map((teamId) => {
                  return (
                    <div
                      className="flex flex-row items-center justify-center text-sm p-2 h-[2.1rem] bg-slate-950 bg-opacity-20 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden"
                      key={teamId}
                    >
                      <StatusCell
                        flags={teamFlagMap[teamId]?.[service.name] ?? {}}
                      />
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default SimpleDisplay;
