import { createLazyFileRoute } from "@tanstack/react-router";
import SimpleDisplay from "../components/SimpleDisplay";
import {
  DUMMY_FLAGSUBMISSION_LOG,
  DUMMY_SCOREBOARD_DATA,
} from "../utils/constants";
import type { FlagType } from "../utils/types";

const BOX = [
  { title: "Executions in queue", value: "26" },
  { title: "Exploits", value: "3" },
  { title: "Flags received", value: "27" },
  { title: "Accepted", value: "395" },
  { title: "Rejected", value: "22" },
  { title: "Duplicates", value: "1" },
];
export const Route = createLazyFileRoute("/")({
  component: () => (
    <main className="flex flex-col gap-3">
      <div className="grid grid-cols-6 gap-3">
        {BOX.map((box) => (
          <div
            key={box.title}
            className="px-3 py-2 shadow-inner bg-slate-950 bg-opacity-30 border-slate-950 border-opacity-20 border-2 rounded-sm flex flex-col"
          >
            <span className="text-xs">{box.title}</span>
            <span className="text-sm font-bold">{box.value}</span>
          </div>
        ))}
      </div>
      <SimpleDisplay
        data={{
          scoreboard: DUMMY_SCOREBOARD_DATA,
          flag: DUMMY_FLAGSUBMISSION_LOG as FlagType[],
        }}
      />
    </main>
  ),
});
