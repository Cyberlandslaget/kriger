import { useAtomValue } from "jotai";
import { servicesAtom, teamFlagStatusAtom, teamsAtom } from "../utils/atoms";
import { StatusCell } from "./StatusCell";

function SimpleDisplay() {
  const services = useAtomValue(servicesAtom);
  const teams = useAtomValue(teamsAtom);
  const teamFlagMap = useAtomValue(teamFlagStatusAtom);

  return (
    <div
      className="flex flex-col h-full relative rounded-md overflow-auto"
    >
      <div className="min-w-full text-sm bg-primary-bg z-10 absolute overflow-x-hidden overflow-y-scroll">
        <div className="flex mb-2">
          <div className="min-w-48 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-left">
            Team
          </div>
          {services.map((service) => (
            <div
              className="w-full min-w-36 ml-2 font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-nowrap sticky"
              key={service.name}
            >
              {service.name}
            </div>
          ))}
        </div>
      </div>

      <div className="flex flex-col min-w-full h-full overflow-x-hidden overflow-y-scroll absolute pt-12">
        {Object.entries(teams).map(([teamId, team]) => (
          <div key={teamId} className="flex mb-2">
            <div
              className="min-w-48 bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2"
              title={`[${teamId}] ${team.name ?? ""}`}
            >
              <div className="w-full flex items-center text-sm p-1.5 h-full shadow-inner  rounded-sm transition-all duration-150 truncate">
                [{teamId}] {team.name}
              </div>
            </div>

            {services.map((service) => (
              <StatusCell
                flags={teamFlagMap[teamId]?.[service.name] ?? {}}
                key={service.name}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

export default SimpleDisplay;
