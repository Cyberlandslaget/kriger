// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import { useCallback, useEffect, useRef } from "react";
import { WebSocketService } from "../services/webSocket";
import type { WebSocketMessage } from "../services/models";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import {
  currentTickAtom,
  exploitExecutionRequestDispatch,
  exploitExecutionResultDispatch,
  exploitExecutionsPurgeDispatch,
  exploitsAtom,
  serverConfigAtom,
  servicesAtom,
  teamFlagPurgeDispatch,
  teamFlagSubmissionDispatch,
  teamsAtom,
} from "./atoms";
import {
  useCompetitionServices,
  useCompetitionTeams,
  useExploitsData,
  useServerConfig,
} from "../services/rest";

export const useWebSocketProvider = (url: string) => {
  const [currentTick, setCurrentTick] = useAtom(currentTickAtom);
  const flagSubmissionDispatch = useSetAtom(teamFlagSubmissionDispatch);
  const flagPurgeDispatch = useSetAtom(teamFlagPurgeDispatch);
  const executionRequestDispatch = useSetAtom(exploitExecutionRequestDispatch);
  const executionResultDispatch = useSetAtom(exploitExecutionResultDispatch);
  const executionPurgeDispatch = useSetAtom(exploitExecutionsPurgeDispatch);
  const serverConfig = useAtomValue(serverConfigAtom);

  const handleMessage = useCallback(
    (event: WebSocketMessage) => {
      if (!serverConfig) return;

      // Ignore messages that are older than what we expect
      // If the currentTick has not been updated yet, we'll consume the event
      // and hope that old data gets purged once the currentTick state is updated
      const oldest =
        new Date(serverConfig.competition.start).getTime() +
        (currentTick - serverConfig.competition.flagValidity + 1) *
          serverConfig.competition.tick *
          1000;

      switch (event.type) {
        case "scheduling_start":
          setCurrentTick(event.tick);
          break;
        case "flag_submission":
        case "flag_submission_result": {
          if (event.published < oldest) {
            break;
          }

          flagSubmissionDispatch(event);
          break;
        }
        case "execution_request":
          if (event.published < oldest) {
            break;
          }

          executionRequestDispatch(event);
          break;
        case "execution_result":
          if (event.published < oldest) {
            break;
          }

          executionResultDispatch(event);
          break;
      }
    },
    [
      serverConfig,
      currentTick,
      setCurrentTick,
      flagSubmissionDispatch,
      executionRequestDispatch,
      executionResultDispatch,
    ],
  );

  const handleMessageRef = useRef<typeof handleMessage>();

  // Peerform pruning reactively upon state changes instead of directly in the message handler
  // to avoid state inconsistency issues.
  useEffect(() => {
    if (!serverConfig || currentTick < 0) return;

    const oldest =
      new Date(serverConfig.competition.start).getTime() +
      (currentTick - serverConfig.competition.flagValidity + 1) *
        serverConfig.competition.tick *
        1000;
    flagPurgeDispatch(oldest);
    executionPurgeDispatch(oldest);
  }, [currentTick, serverConfig, flagPurgeDispatch, executionPurgeDispatch]);

  useEffect(() => {
    handleMessageRef.current = handleMessage;
  }, [handleMessage]);

  // We create a new WS connection when the competition config has been updated or first received.
  // Consumers are expected to handle websocket events idempotently.
  useEffect(() => {
    if (!serverConfig) return;

    // We add 1 to the flag validity here to account for the moving time window
    const service = new WebSocketService(
      url,
      () =>
        Date.now() -
        serverConfig.competition.tick *
          (serverConfig.competition.flagValidity + 1) *
          1000,
      (message) => {
        handleMessageRef.current?.(message);
      },
    );
    return () => service.close();
  }, [url, serverConfig]);
};

export const useConfigProvider = () => {
  const setServerConfig = useSetAtom(serverConfigAtom);
  const { data } = useServerConfig();
  useEffect(() => {
    if (!data) return;

    const { data: config } = data;
    setServerConfig(config);
  }, [data, setServerConfig]);
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

export const useExploits = () => {
  const setExploits = useSetAtom(exploitsAtom);

  const { data: exploits } = useExploitsData();

  useEffect(() => {
    if (exploits?.data) {
      setExploits(
        exploits?.data?.sort(
          (a, b) =>
            a.manifest.service.localeCompare(b.manifest.service) +
            a.manifest.name.localeCompare(b.manifest.name),
        ),
      );
    }
  }, [exploits, setExploits]);
};
