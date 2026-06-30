import { env } from "#/env.ts";

export type ErrorResponse = {
  error?: string;
  message?: string;
};

export const API_BASE_URL = env.VITE_SERVER_URL ?? "http://127.0.0.1:7150/api";

export async function getErrorResponse(response: Response, fallback = "Request failed") {
  try {
    const body = (await response.json()) as ErrorResponse;
    return body.error ?? body.message ?? fallback;
  } catch {
    return fallback;
  }
}
