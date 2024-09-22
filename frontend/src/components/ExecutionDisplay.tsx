import { useAtomValue } from "jotai";
import { executionsAtom, executionStatusAggregateAtom } from "../utils/atoms";
import { useMemo } from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { forwardRef } from "react";
import { FixedSizeList as List } from "react-window";
import { ExecutionResultStatusCode } from "../utils/enums";
import clsx from "clsx";

function ExecutionDisplay() {
  const executionStatusAggregate = useAtomValue(executionStatusAggregateAtom);
  const executionMap = useAtomValue(executionsAtom);

  const executions = useMemo(() => {
    // TODO: add filtering here
    return executionMap.executions;
  }, [executionMap]);

  return (
    <>
      <div className="flex items-center justify-end w-full text-sm text-slate-300">
        <p>
          {executionMap.sortedSequence.length} executions |{" "}
          {executionStatusAggregate.count} aggregated |{" "}
          {executionStatusAggregate.pendingCount} pending
        </p>
      </div>
      <div className="flex flex-col h-full relative rounded-md">
        <AutoSizer>
          {({ height, width }) => (
            <List
              height={height}
              itemCount={executionMap.sortedSequence.length}
              itemSize={48}
              width={width}
              innerElementType={forwardRef(({ children, ...rest }, ref) => (
                <table ref={ref} {...rest} className="relative">
                  <thead className="sticky top-0 bg-primary-bg h-10 z-10">
                    <tr className="flex mb-2 gap-2 text-left">
                      <th className="min-w-24 max-w-48 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-right">
                        Sequence
                      </th>
                      <th className="min-w-24 max-w-48 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-right">
                        Team Id
                      </th>
                      <th className="min-w-48 w-full h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm">
                        Exploit name
                      </th>
                      <th className="min-w-24 max-w-48 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-right">
                        Status
                      </th>
                      <th className="min-w-24 max-w-48 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-right">
                        Time
                      </th>
                      <th className="min-w-36 max-w-48 h-10 items-center font-bold p-2 shadow-inner bg-slate-950/30 border-slate-950 border-opacity-20 border-2 rounded-sm text-right">
                        Published
                      </th>
                    </tr>
                  </thead>
                  <tbody>{children}</tbody>
                </table>
              ))}
            >
              {({ index, style }) => {
                const sequence = executionMap.sortedSequence[index];
                const execution = executions[sequence];
                if (!execution) return <></>;
                const borderColor =
                  execution?.status === ExecutionResultStatusCode.Success
                    ? "border-green-500"
                    : execution?.status === ExecutionResultStatusCode.Timeout
                      ? "border-amber-200/40"
                      : execution?.status === ExecutionResultStatusCode.Error
                        ? "border-red-500/50"
                        : "border-slate-950";
                return (
                  <tr
                    key={`key-${index}`}
                    style={{ ...style }}
                    className={clsx(
                      "flex min-w-full !h-10 mt-12 gap-2 bg-slate-950 bg-opacity-30 border-opacity-20 border-2 text-sm rounded-sm",
                      borderColor,
                    )}
                  >
                    <td className="min-w-24 max-w-48 h-10">
                      <div className="w-full flex items-center justify-end p-1.5 h-full">
                        <p className="truncate">{execution.sequence}</p>
                      </div>
                    </td>
                    <td className="min-w-24 max-w-48 h-10">
                      <div className="w-full flex items-center justify-end p-1.5 h-full">
                        <p className="truncate">{execution.teamId}</p>
                      </div>
                    </td>
                    <td className="min-w-48 w-full h-10">
                      <div className="w-full flex items-center text-sm p-1.5 h-full shadow-inner  rounded-sm transition-all duration-150 truncate">
                        {execution.exploitName}
                      </div>
                    </td>
                    <td className="min-w-24 max-w-48 h-10">
                      <div className="w-full flex items-center justify-end p-1.5 h-full">
                        {execution?.status ? (
                          <p className="truncate">
                            {ExecutionResultStatusCode[execution.status]}
                          </p>
                        ) : (
                          <></>
                        )}
                      </div>
                    </td>
                    <td className="min-w-24 max-w-48 h-10">
                      <div className="w-full flex items-center justify-end p-1.5 h-full">
                        {execution?.time ? (
                          <p className="truncate">
                            {Math.floor((execution?.time ?? 0) / 1000)}s
                          </p>
                        ) : (
                          <></>
                        )}
                      </div>
                    </td>
                    <td className="min-w-36 max-w-48 h-8">
                      <div className="w-full flex items-center justify-end p-1.5 h-full">
                        <p className="truncate">{execution.published}</p>
                      </div>
                    </td>
                  </tr>
                );
              }}
            </List>
          )}
        </AutoSizer>
      </div>
    </>
  );
}

export default ExecutionDisplay;
