import { useAtomValue } from "jotai";
import { servicesAtom, teamFlagStatusAtom, teamsAtom } from "../utils/atoms";
import { StatusCell } from "./StatusCell";
import { useEffect, useRef } from "react";

function SimpleDisplay() {
  const services = useAtomValue(servicesAtom);
  const teams = useAtomValue(teamsAtom);
  const teamFlagMap = useAtomValue(teamFlagStatusAtom);
  const divRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleResize = () => {
      if (divRef.current) {
        const boundingRect = divRef.current.getBoundingClientRect();
        const maxHeight = window.innerHeight - boundingRect.top - 16;
        divRef.current.style.height = `${maxHeight}px`;
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  return (
    <div
      ref={divRef}
      className="flex flex-col h-screen relative rounded-md overflow-auto"
    >
      <div className="min-w-full text-sm bg-primary-bg z-10 overflow-x-hidden overflow-y-scroll absolute">
        <div className="flex mb-2">
          <div className="min-w-48 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-left">
            Team
          </div>
          {services.map((service) => (
            <div
              className="w-full min-w-32 ml-2 font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-nowrap sticky"
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
