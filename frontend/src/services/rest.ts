import useSWR from "swr";
import { CONFIG } from "../utils/constants";
import type {
  APIErrorResponse,
  APISuccessResponse,
  CompetitionConfig,
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

export const useCompetitionConfig = () =>
  useSWR<APISuccessResponse<CompetitionConfig>, APIErrorResponse>(
    "/config/competition",
    fetcher,
  );
