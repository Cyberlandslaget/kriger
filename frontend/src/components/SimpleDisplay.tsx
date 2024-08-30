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
        <table className="border-spacing-2 border-separate">
          <thead className="text-sm">
            <tr>
              <th className="min-w-48 items-center font-bold shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-left">
                Team
              </th>
              {services.map((service) => (
                <th
                  className="font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm"
                  key={service.name}
                >
                  {service.name}
                </th>
              ))}
            </tr>
          </thead>

          <tbody>
            {Object.entries(teams).map(([teamId, team]) => (
              <tr key={teamId}>
                <td
                  className="bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2"
                  title={`[${teamId}] ${team.name ?? ""}`}
                >
                  <div className="flex items-center text-sm p-1.5 h-full shadow-inner  rounded-sm transition-all duration-150 truncate">
                    [{teamId}] {team.name}
                  </div>
                </td>

                {services.map((service) => (
                  <StatusCell
                    flags={teamFlagMap[teamId]?.[service.name] ?? {}}
                    key={service.name}
                  />
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default SimpleDisplay;
