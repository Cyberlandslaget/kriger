import { createLazyFileRoute } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { configurationAtom } from "../utils/atoms";
import { type ChangeEvent, useEffect } from "react";

export const Route = createLazyFileRoute("/config")({
  component: () => Configuration(),
});

function Configuration() {
  const [configuration, setConfiguration] = useAtom(configurationAtom);

  const onChangeHandler = (
    event: ChangeEvent<HTMLInputElement>,
    key: string,
  ) => {
    const value = event.target.value;
    setConfiguration((prev) => {
      return { ...prev, [key]: value };
    });
  };

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      localStorage.setItem(
        "kriger-configuration",
        JSON.stringify(configuration),
      );
    }, 500);
    return () => clearTimeout(timeoutId);
  }, [configuration]);

  return (
    <main className="flex flex-col gap-3">
      <div>
        <div className="grid grid-cols-2 gap-2 items-center p-2 rounded-md">
          <h2 className="text-slate-300">Flag regex:</h2>
          <input
            className="w-full bg-slate-950/80 px-2 py-1 text-slate-300 rounded-sm"
            type="text"
            onChange={(e) => onChangeHandler(e, "flagRegex")}
            value={configuration.flagRegex}
          />
        </div>
        <div className="grid grid-cols-2 gap-2 items-center p-2 rounded-md">
          <h2 className="text-slate-300">Minutes to fetch:</h2>
          <input
            className="w-full bg-slate-950/80 px-2 py-1 text-slate-300 rounded-sm"
            type="number"
            onChange={(e) => onChangeHandler(e, "minutesToFetch")}
            value={configuration.minutesToFetch}
          />
        </div>
      </div>
    </main>
  );
}
export default Configuration;
