import { useCallback, useEffect, useRef } from "react";
import { WebSocketService } from "../services/webSocket";
import type { WebSocketMessage } from "../services/models";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
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
  const [currentTick, setCurrentTick] = useAtom(currentTickAtom);
  const flagSubmissionDispatch = useSetAtom(teamFlagSubmissionDispatch);
  const flagPurgeDispatch = useSetAtom(teamFlagPurgeDispatch);
  const competitionConfig = useAtomValue(competitionConfigAtom);

  const handleMessage = useCallback(
    (event: WebSocketMessage) => {
      if (!competitionConfig) return;

      switch (event.type) {
        case "scheduling_start":
          setCurrentTick(event.tick);
          break;
        case "flag_submission":
        case "flag_submission_result": {
          // TODO: Investigate and check if the message timing is working as expected and that everything is aggregated properly

          // Ignore messages that are older than what we expect
          // If the currentTick has not been updated yet, we'll consume the event
          // and hope that old data gets purged once the currentTick state is updated
          const oldest =
            new Date(competitionConfig.start).getTime() +
            (currentTick - competitionConfig.flagValidity + 1) *
              competitionConfig.tick *
              1000;
          if (event.published < oldest) {
            break;
          }

          flagSubmissionDispatch(event);
          break;
        }
      }
    },
    [currentTick, competitionConfig, setCurrentTick, flagSubmissionDispatch],
  );

  const handleMessageRef = useRef<typeof handleMessage>();

  // Peerform pruning reactively upon state changes instead of directly in the message handler
  // to avoid state inconsistency issues.
  useEffect(() => {
    if (!competitionConfig || currentTick < 0) return;

    const oldest =
      new Date(competitionConfig.start).getTime() +
      (currentTick - competitionConfig.flagValidity + 1) *
        competitionConfig.tick *
        1000;
    flagPurgeDispatch(oldest);
  }, [currentTick, competitionConfig, flagPurgeDispatch]);

  useEffect(() => {
    handleMessageRef.current = handleMessage;
  }, [handleMessage]);

  // We create a new WS connection when the competition config has been updated or first received.
  // Consumers are expected to handle websocket events idempotently.
  useEffect(() => {
    if (!competitionConfig) return;

    // We add 1 to the flag validity here to account for the moving time window
    const service = new WebSocketService(
      url,
      () =>
        Date.now() -
        competitionConfig.tick * (competitionConfig.flagValidity + 1) * 1000,
      (message) => {
        handleMessageRef.current?.(message);
      },
    );
    return () => service.close();
  }, [url, competitionConfig]);
};

export const useConfigProvider = () => {
  const setCompetitionConfig = useSetAtom(competitionConfigAtom);
  const { data } = useCompetitionConfig();
  useEffect(() => {
    if (!data) return;

    const { data: config } = data;
    setCompetitionConfig(config);
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
