import { createLazyFileRoute } from "@tanstack/react-router";
import { FLAG_CODE } from "../utils/enums";

export const Route = createLazyFileRoute("/submit")({
  component: () => (
    <main className="flex flex-col gap-6">
      {/* Textarea to dump texts containing one or many flags */}
      <textarea
        className="w-full min-h-48 bg-slate-950/80 p-2 text-slate-300 rounded-sm"
        name="flags"
        id="flags"
      />

      {/* Textarea onChange-events should find and match on flag regex, updating the count number  */}
      <div className="flex justify-between">
        <div>
          <p className="flex items-center gap-2">
            <span className="text-slate-300">Matched flags:</span> 2
          </p>
        </div>
        <div className="flex gap-3">
          <button
            type="button"
            className="flex items-center justify-center gap-2 bg-red-500/80 text-center truncate p-1 px-3 rounded-sm transition-all hover:!bg-red-600/60"
          >
            Push to queue
          </button>
          <button
            type="button"
            className="flex items-center justify-center gap-2 bg-red-500/80 text-center truncate p-1 px-3 rounded-sm transition-all hover:!bg-red-600/60"
          >
            Submit flags
          </button>
        </div>
      </div>

      {/* Table listing the flags alongside the responses after being submitted */}
      <table>
        <thead>
          <tr className="text-left border-b-8 border-transparent">
            <th>Status</th>
            <th>Value</th>
            <th>Response</th>
          </tr>
        </thead>
        <tbody className="text-sm">
          <tr className="text-left pt-2">
            <td>{FLAG_CODE.OK}</td>
            <td>ECSC_KcpNAJ2gTzNviLzZE6hsIKIEJqbV4Dcr</td>
            <td>Accepted: X flag points</td>
          </tr>
          <tr className="text-left">
            <td>{FLAG_CODE.OK}</td>
            <td>ECSC_KcpNAJ2gTzNviLzZE6hsIKIEJqbV4Dcr</td>
            <td>Accepted: X flag points</td>
          </tr>
        </tbody>
      </table>
    </main>
  ),
});
