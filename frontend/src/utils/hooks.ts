import { useCallback, useEffect, useRef } from "react";
import { WebSocketService } from "../services/webSocket";
import type { WebSocketMessage } from "../services/models";
import { useSetAtom } from "jotai";
import { currentTickAtom } from "./atoms";

export const useWebSocket = (url: string) => {
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
