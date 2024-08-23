import useSWR from "swr";
import { CONFIG } from "../utils/constants";
import type {
  APIErrorResponse,
  APISuccessResponse,
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
