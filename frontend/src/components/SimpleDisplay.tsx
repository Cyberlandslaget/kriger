import { useAtomValue } from "jotai";
import { currentTickAtom } from "../utils/atoms";
import { FLAG_CODE, SERVICE_STATUS } from "../utils/enums";
import type { FlagSubmissionResult, ScoreboardType } from "../utils/types";
import { useMemo } from "react";
import { removeSimpleDuplicates } from "../utils/removeSimpleDuplicates";

type Data = {
  [key: number]: FlagSubmissionResult;
};
type SimpleOverviewProps = {
  data: Data;
  status: number;
  currentTick: number;
  total: number;
};
const StatusRow = ({
  data,
  status: _,
  currentTick,
  total,
}: SimpleOverviewProps) => {
  const status = Object.values(data).splice(
    Math.max(currentTick - total, 0),
    currentTick - total > 0 ? total : currentTick,
  );

  return (
    <>
      {status.map((log, index) => (
        <span key={Math.random()} className="flex items-center">
          {getStatusIcon(log, currentTick - total + index + 1)}
        </span>
      ))}
    </>
  );
};

type SimpleDisplayProps = {
  data: {
    scoreboard: ScoreboardType;
    flag: FlagSubmissionResult[];
  };
};
function SimpleDisplay({ data }: SimpleDisplayProps) {
  const currentTick = 5 ?? useAtomValue(currentTickAtom);
  const total = 5;

  const teams = useMemo(
    () => Object.entries(data?.scoreboard?.teams),
    [data?.scoreboard?.teams],
  );
  const services = useMemo(
    () => Object.keys(Object.values(data.scoreboard.teams)[0].services),
    [data?.scoreboard?.teams],
  );

  const filteredStatusData = useMemo(() => {
    return removeSimpleDuplicates(
      Object.keys(data.scoreboard.teams),
      services,
      data.flag,
      currentTick,
    );
  }, [currentTick, data.flag, data.scoreboard.teams, services]);

  return (
    <div className="relative wrapper my-1 gap-2 h-[calc(100%-1.5rem)]">
      <div className="w-full h-full rounded-md overflow-auto">
        <div className="grid [grid-template-columns:13rem_1fr] gap-2 min-h-0">
          <div className="flex flex-col gap-1">
            <p className="flex mb-1 items-center font-bold text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden">
              Team
            </p>
            {teams.map((team, _teamIndex) => (
              <div
                key={`team_${team[0]}_overview`}
                id={team[0]}
                className="flex items-center text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm transition-all duration-150"
                title={`[${team[0]}] ${team[1].name ?? ""}`}
              >
                <p className="truncate">
                  [{team[0]}] {team[1].name}
                </p>
              </div>
            ))}
          </div>

          <div className="flex gap-1 overflow-auto w-full pb-1">
            {services.map((service) => (
              <div
                key={`service_${service}_overview`}
                className="flex flex-col gap-1"
              >
                <p className="flex w-44 mb-1 items-center justify-center font-bold text-sm p-2 h-[2.1rem] shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden transition-all duration-150">
                  {service}
                </p>
                {teams.map((team) => {
                  return (
                    <p
                      key={`service_${team[0]}_overview`}
                      className={`flex flex-row items-center justify-center text-sm p-2 h-[2.1rem] bg-slate-950 bg-opacity-20 border-slate-950 border-opacity-20 border-2 rounded-sm text-ellipsis whitespace-nowrap overflow-hidden ${
                        team[1].services[service] === SERVICE_STATUS.OK
                          ? ""
                          : team[1].services[service] === SERVICE_STATUS.DOWN
                            ? "border-red-400 border-opacity-50"
                            : "border-yellow-400 border-opacity-50"
                      }`}
                    >
                      <StatusRow
                        data={filteredStatusData[team[0]][service] as Data}
                        status={team[1].services[service]}
                        currentTick={currentTick}
                        total={total}
                      />
                    </p>
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

function getStatusIcon(chall: FlagSubmissionResult, tick: number) {
  switch (chall?.status) {
    case FLAG_CODE.Ok:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-green text-green-dark rounded-sm"
        >
          <title>OK ({tick})</title>
          <path
            fill="currentColor"
            d="M470.6 105.4c12.5 12.5 12.5 32.8 0 45.3l-256 256c-12.5 12.5-32.8 12.5-45.3 0l-128-128c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0L192 338.7 425.4 105.4c12.5-12.5 32.8-12.5 45.3 0z"
          />
        </svg>
      );
    case FLAG_CODE.Duplicate:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-yellow text-yellow-dark rounded-sm"
        >
          <title>DUP ({tick})</title>
          <path
            fill="currentColor"
            d="M105.1 202.6c7.7-21.8 20.2-42.3 37.8-59.8c62.5-62.5 163.8-62.5 226.3 0L386.3 160H336c-17.7 0-32 14.3-32 32s14.3 32 32 32H463.5c0 0 0 0 0 0h.4c17.7 0 32-14.3 32-32V64c0-17.7-14.3-32-32-32s-32 14.3-32 32v51.2L414.4 97.6c-87.5-87.5-229.3-87.5-316.8 0C73.2 122 55.6 150.7 44.8 181.4c-5.9 16.7 2.9 34.9 19.5 40.8s34.9-2.9 40.8-19.5zM39 289.3c-5 1.5-9.8 4.2-13.7 8.2c-4 4-6.7 8.8-8.1 14c-.3 1.2-.6 2.5-.8 3.8c-.3 1.7-.4 3.4-.4 5.1V448c0 17.7 14.3 32 32 32s32-14.3 32-32V396.9l17.6 17.5 0 0c87.5 87.4 229.3 87.4 316.7 0c24.4-24.4 42.1-53.1 52.9-83.7c5.9-16.7-2.9-34.9-19.5-40.8s-34.9 2.9-40.8 19.5c-7.7 21.8-20.2 42.3-37.8 59.8c-62.5 62.5-163.8 62.5-226.3 0l-.1-.1L125.6 352H176c17.7 0 32-14.3 32-32s-14.3-32-32-32H48.4c-1.6 0-3.2 .1-4.8 .3s-3.1 .5-4.6 1z"
          />
        </svg>
      );
    case FLAG_CODE.Old:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-yellow text-yellow-dark rounded-sm"
        >
          <title>OLD ({tick})</title>
          <path
            fill="currentColor"
            d="M232 120C232 106.7 242.7 96 256 96C269.3 96 280 106.7 280 120V243.2L365.3 300C376.3 307.4 379.3 322.3 371.1 333.3C364.6 344.3 349.7 347.3 338.7 339.1L242.7 275.1C236 271.5 232 264 232 255.1L232 120zM256 0C397.4 0 512 114.6 512 256C512 397.4 397.4 512 256 512C114.6 512 0 397.4 0 256C0 114.6 114.6 0 256 0zM48 256C48 370.9 141.1 464 256 464C370.9 464 464 370.9 464 256C464 141.1 370.9 48 256 48C141.1 48 48 141.1 48 256z"
          />
        </svg>
      );
    case FLAG_CODE.Invalid:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 320 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-red text-red-dark rounded-sm"
        >
          <title>INV ({tick})</title>
          <path
            fill="currentColor"
            d="M310.6 150.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L160 210.7 54.6 105.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3L114.7 256 9.4 361.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L160 301.3 265.4 406.6c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L205.3 256 310.6 150.6z"
          />
        </svg>
      );
    case FLAG_CODE.Error:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 128 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-red text-red-dark rounded-sm"
        >
          <title>ERR ({tick})</title>
          <path
            fill="currentColor"
            d="M96 64c0-17.7-14.3-32-32-32S32 46.3 32 64V320c0 17.7 14.3 32 32 32s32-14.3 32-32V64zM64 480c22.1 0 40-17.9 40-40s-17.9-40-40-40s-40 17.9-40 40s17.9 40 40 40z"
          />
        </svg>
      );
    case FLAG_CODE.Own:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 128 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-red text-red-dark rounded-sm"
        >
          <title>OWN ({tick})</title>
          <path
            fill="currentColor"
            d="M96 64c0-17.7-14.3-32-32-32S32 46.3 32 64V320c0 17.7 14.3 32 32 32s32-14.3 32-32V64zM64 480c22.1 0 40-17.9 40-40s-17.9-40-40-40s-40 17.9-40 40s17.9 40 40 40z"
          />
        </svg>
      );
    default:
      return (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 320 512"
          className="w-[1.15rem] h-[1.15rem] p-[1.5px] mx-[3px] bg-white/80 transparent rounded-sm"
        >
          <title>NONE ({tick})</title>
        </svg>
      );
  }
}
