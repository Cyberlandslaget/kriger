// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

import useSWR from "swr";
import { CONFIG } from "../utils/constants";
import type {
  APIErrorResponse,
  APISuccessResponse,
  Exploit,
  ServerConfig,
  Service,
  Team,
} from "./models";

const fetcher = async <T>(path: string): Promise<APISuccessResponse<T>> => {
  const res = await fetch(CONFIG.restUrl + path);
  if (!res.ok) {
    try {
      throw await res.json();
    } catch (error) {
      throw {
        error: {
          message: `Parsing error: ${error}`,
        },
      } as APIErrorResponse;
    }
  }

  try {
    return await res.json();
  } catch (error) {
    throw {
      error: {
        message: `Parsing error: ${error}`,
      },
    } as APIErrorResponse;
  }
};

export const useServerConfig = () =>
  useSWR<APISuccessResponse<ServerConfig>, APIErrorResponse>(
    "/config/server",
    fetcher,
  );

export const useCompetitionServices = () =>
  useSWR<APISuccessResponse<Service[]>, APIErrorResponse>(
    "/competition/services",
    fetcher,
  );

export const useCompetitionTeams = () =>
  useSWR<APISuccessResponse<Record<string, Team>>, APIErrorResponse>(
    "/competition/teams",
    fetcher,
  );

export const useExploitsData = () =>
  useSWR<APISuccessResponse<Exploit[]>, APIErrorResponse>(
    "/exploits",
    fetcher,
  );

export const executeExploit = async (exploit: Exploit): Promise<unknown> => {
  const response = await fetch(`${CONFIG.restUrl}/exploits/${exploit.manifest.name}/execute`, {
    method: 'POST',
  });
  return await response.json();
};

export const updateExploit = async (exploit: Exploit): Promise<unknown> => {
  const response = await fetch(`${CONFIG.restUrl}/exploits/${exploit.manifest.name}`, {
    method: 'PUT',
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(exploit)
  });
  return await response.json();
};