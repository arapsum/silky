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

type ApiRequestOptions = RequestInit & {
  fallback?: string;
};

export async function apiRequest<T>(
  path: string,
  { fallback = "Request failed", headers, body, ...init }: ApiRequestOptions = {},
): Promise<T> {
  const requestHeaders = new Headers(headers);
  requestHeaders.set("Accept", "application/json");

  if (body && !(body instanceof FormData) && !requestHeaders.has("Content-Type")) {
    requestHeaders.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    body,
    headers: requestHeaders,
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error(await getErrorResponse(response, fallback));
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}
