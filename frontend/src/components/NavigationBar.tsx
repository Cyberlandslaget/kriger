import { Link } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { serverConfigAtom, statusAtom } from "../utils/atoms";
import { useInterval } from "usehooks-ts";
import { useMemo, useState } from "react";
import clsx from "clsx";

const ROUTES = [
  { href: "/", text: "Dashboard" },
  { href: "/executions", text: "Executions" },
  { href: "/exploits", text: "Exploits" },
  { href: "/submit", text: "Manual submit" },
  // { href: "/config", text: "Configuration" },
];

function NavigationBar() {
  const status = useAtomValue(statusAtom);
  const serverConfig = useAtomValue(serverConfigAtom);

  const startTime = useMemo(
    () =>
      serverConfig
        ? new Date(serverConfig.competition.start).getTime()
        : undefined,
    [serverConfig],
  );
  const tickStart = useMemo(
    () =>
      startTime && serverConfig
        ? startTime + status.currentTick * serverConfig.competition.tick * 1000
        : undefined,
    [startTime, status.currentTick, serverConfig],
  );
  const [currentTime, setCurrentTime] = useState<number | undefined>(tickStart);

  // Values in the range [0, inf). Values below 1 represents tick progress.
  // Values greater than or equal to 1 represents ticks that are waiting for the server.
  const tickProgress = useMemo(
    () =>
      currentTime && tickStart && serverConfig
        ? Math.max(
            (currentTime - tickStart) / serverConfig.competition.tick / 1000,
            0,
          )
        : 0,
    [currentTime, tickStart, serverConfig],
  );

  const timeUntilNextTick = useMemo(() => {
    return tickProgress && serverConfig
      ? (1 - tickProgress) * serverConfig.competition.tick
      : undefined;
  }, [tickProgress, serverConfig]);

  // JavaScript timers are inaccurate by nature
  useInterval(() => {
    setCurrentTime(Date.now());
  }, 50);

  return (
    <nav>
      {/* Progress bar for a tick */}
      <div className="bg-slate-400/20 h-2">
        {/* FIXME: This progress bar is NOT accurate due to how the animation easing works. */}
        <div
          className="h-full bg-red-400/80 transition-[width] duration-50 ease-linear"
          style={{ width: `${Math.min(tickProgress * 100, 100)}%` }}
        />
      </div>

      <div className="p-6 flex flex-row items-center justify-between gap-6">
        <div className="flex items-center gap-6">
          <Link to={"/"}>
            <div className="text-xl font-bold">Kriger</div>
          </Link>
          {ROUTES.map((link) => (
            <Link
              key={link.href}
              to={link.href}
              activeProps={{
                className: "font-bold !opacity-100",
              }}
              className="opacity-60 hidden md:block"
            >
              {link.text}
            </Link>
          ))}
        </div>

        {/* Current tick + remaining tick time */}
        <div className="font-bold">
          {/* Highlight the tick as red if the tickProgress is > 1. This means that the server is not delivering on time. */}
          <span className={clsx(tickProgress > 1 && "text-red-500")}>
            Tick {status.currentTick}
          </span>{" "}
          {/* We don't support end round yet */}
          {/* / {status.rounds}{" "} */}
          <span className="font-normal text-slate-300">
            ({timeUntilNextTick?.toFixed(0) ?? "∞"}s)
          </span>
        </div>
      </div>
    </nav>
  );
}
export default NavigationBar;
