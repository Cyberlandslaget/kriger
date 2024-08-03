import { useCallback, useEffect, useRef } from "react";
import { WebSocketService } from "../services/webSocket";
import type { WebSocketMessage } from "../services/models";
import { useSetAtom } from "jotai";
import { competitionConfigAtom, currentTickAtom } from "./atoms";
import { useCompetitionConfig } from "../services/rest";

export const useWebSocketProvider = (url: string) => {
  const setCurrentTick = useSetAtom(currentTickAtom);

  const handleMessage = useCallback(
    (event: WebSocketMessage) => {
      switch (event.type) {
        case "scheduling_start":
          setCurrentTick(event.tick);
          break;
      }
    },
    [setCurrentTick],
  );

  const handleMessageRef = useRef<typeof handleMessage>();

  useEffect(() => {
    handleMessageRef.current = handleMessage;
  }, [handleMessage]);

  useEffect(() => {
    const service = new WebSocketService(url, (message) => {
      handleMessageRef.current?.(message);
    });
    return () => service.close();
  }, [url]);
};

export const useConfigProvider = () => {
  const setCompetitionConfig = useSetAtom(competitionConfigAtom);
  const { data } = useCompetitionConfig();

  useEffect(() => {
    if (!data) return;

    const { data: config } = data;
    setCompetitionConfig({
      start: config.start,
      tick: config.tick,
      flagFormat: config.flag_format,
    });
  }, [data, setCompetitionConfig]);
};
