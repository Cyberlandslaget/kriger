import { Link } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { currentTickAtom, statusAtom } from "../utils/atoms";

const ROUTES = [
  { href: "/", text: "Dashboard" },
  { href: "/executions", text: "Executions" },
  { href: "/exploits", text: "Exploits" },
  { href: "/submit", text: "Manual submit" },
  { href: "/config", text: "Configuration" },
];

function NavigationBar() {
  const currentTick = useAtomValue(currentTickAtom);
  const status = useAtomValue(statusAtom);

  return (
    <nav>
      {/* Progress bar for a tick */}
      <div className="bg-slate-400/20 h-2">
        <div className="h-full w-[30%] bg-red-400/80" />
      </div>

      <div className="p-6 flex flex-row items-center justify-between gap-6">
        <div className="flex items-center gap-6 ">
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
              className="opacity-60"
            >
              {link.text}
            </Link>
          ))}
        </div>

        {/* Current tick + remaining tick time */}
        <div className="font-bold">
          Tick {currentTick} / {status.rounds}{" "}
          <span className="font-normal text-slate-300">(20s)</span>
        </div>
      </div>
    </nav>
  );
}
export default NavigationBar;
