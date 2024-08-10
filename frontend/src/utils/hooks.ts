import { useCallback, useEffect, useRef } from "react";
import { WebSocketService } from "../services/webSocket";
import type { WebSocketMessage } from "../services/models";
import { useAtomValue, useSetAtom } from "jotai";
import {
  competitionConfigAtom,
  currentTickAtom,
  servicesAtom,
  teamFlagPurgeDispatch,
  teamFlagSubmissionDispatch,
  teamsAtom,
} from "./atoms";
import {
  useCompetitionConfig,
  useCompetitionServices,
  useCompetitionTeams,
} from "../services/rest";

export const useWebSocketProvider = (url: string) => {
  const setCurrentTick = useSetAtom(currentTickAtom);
  const flagSubmissionDispatch = useSetAtom(teamFlagSubmissionDispatch);
  const flagPurgeDispatch = useSetAtom(teamFlagPurgeDispatch);
  const competitionConfig = useAtomValue(competitionConfigAtom);

  const handleMessage = useCallback(
    (event: WebSocketMessage) => {
      switch (event.type) {
        case "scheduling_start":
          setCurrentTick(event.tick);
          flagPurgeDispatch(
            new Date(competitionConfig.start).getTime() +
              (event.tick - competitionConfig.flagValidity + 1) *
                competitionConfig.tick *
                1000,
          );
          break;
        case "flag_submission":
        case "flag_submission_result":
          flagSubmissionDispatch(event);
          break;
      }
    },
    [
      setCurrentTick,
      flagSubmissionDispatch,
      flagPurgeDispatch,
      competitionConfig.start,
      competitionConfig.tick,
      competitionConfig.flagValidity,
    ],
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
      flagValidity: config.flag_validity,
      flagFormat: config.flag_format,
    });
  }, [data, setCompetitionConfig]);
};

export const useCompetition = () => {
  const setTeams = useSetAtom(teamsAtom);
  const setServices = useSetAtom(servicesAtom);

  const { data: teams } = useCompetitionTeams();
  const { data: services } = useCompetitionServices();

  useEffect(() => {
    if (teams?.data) {
      setTeams(teams?.data);
    }
  }, [teams, setTeams]);

  useEffect(() => {
    if (services?.data) {
      setServices(services?.data?.sort((a, b) => a.name.localeCompare(b.name)));
    }
  }, [services, setServices]);
};
