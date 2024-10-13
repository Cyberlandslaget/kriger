// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { Link } from "@tanstack/react-router";
import clsx from "clsx";
import { useAtomValue } from "jotai";
import { useMemo, useState } from "react";
import { useInterval } from "usehooks-ts";
import { serverConfigAtom, statusAtom } from "../utils/atoms";

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
  const tickOffset = useMemo(
    () =>
      serverConfig
        ? serverConfig.competition.tickStart *
          serverConfig.competition.tick *
          1000
        : undefined,
    [serverConfig],
  );
  const [currentTime, setCurrentTime] = useState<number | undefined>(tickStart);

  // Values in the range [0, inf). Values below 1 represents tick progress.
  // Values greater than or equal to 1 represents ticks that are waiting for the server.
  const tickProgress = useMemo(
    () =>
      currentTime && tickStart && tickOffset !== undefined && serverConfig
        ? Math.max(
            (currentTime - tickStart + tickOffset) /
              serverConfig.competition.tick /
              1000,
            0,
          )
        : 0,
    [currentTime, tickStart, tickOffset, serverConfig],
  );

  const timeUntilNextTick = useMemo(() => {
    return tickProgress !== undefined && serverConfig
      ? (1 - tickProgress) * serverConfig.competition.tick
      : undefined;
  }, [tickProgress, serverConfig]);

  const [hasTickNotStarted, stringTimeUntilFirstTick] = useMemo(() => {
    // Calculate days, hours, minutes, and seconds
    const remainingTime = (startTime ?? 0) - (currentTime ?? 0);
    const days = Math.floor(remainingTime / 86400000)
      .toString()
      .padStart(2, "0");
    const hours = Math.floor((remainingTime % 86400000) / 3600000)
      .toString()
      .padStart(2, "0");
    const minutes = Math.floor((remainingTime % 3600000) / 60000)
      .toString()
      .padStart(2, "0");
    const seconds = Math.floor((remainingTime % 60000) / 1000)
      .toString()
      .padStart(2, "0");
    return [
      currentTime && startTime && currentTime < startTime,
      `${days}:${hours}:${minutes}:${seconds}`,
    ];
  }, [startTime, currentTime]);

  // JavaScript timers are inaccurate by nature
  useInterval(() => {
    setCurrentTime(Date.now());
  }, 50);

  return (
    <nav className="h-20">
      {/* Progress bar for a tick */}
      <div className="bg-slate-400/20 h-2">
        {/* FIXME: This progress bar is NOT accurate due to how the animation easing works. */}
        <div
          className="h-full bg-red-400/80 transition-[width] duration-50 ease-linear"
          style={{ width: `${Math.min(tickProgress * 100, 100)}%` }}
        />
      </div>

      <div className="p-6 flex flex-row items-center gap-6">
        <div className="flex flex-1 items-center gap-6">
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
        {!hasTickNotStarted ? (
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
        ) : (
          <div className="font-bold">
            <span>Starting in {stringTimeUntilFirstTick}</span>
          </div>
        )}
      </div>
    </nav>
  );
}
export default NavigationBar;
