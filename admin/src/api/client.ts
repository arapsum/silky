import { env } from "#/env.ts";

export type ErrorResponse = {
  error?: string;
  message?: string;
};

export const API_BASE_URL = env.VITE_SERVER_URL ?? "http://127.0.0.1:7150/api";
const EXPIRED_SESSION_MESSAGE = "Expired session";
const MISSING_CREDENTIALS_MESSAGE = "Missing credentials";
const REFRESH_SESSION_PATH = "/auth/refresh";

type SessionExpiredHandler = () => void | Promise<void>;

let refreshSessionPromise: Promise<void> | undefined;
let sessionExpiredHandler: SessionExpiredHandler | undefined;

export function setSessionExpiredHandler(handler: SessionExpiredHandler | undefined) {
  sessionExpiredHandler = handler;

  return () => {
    if (sessionExpiredHandler === handler) {
      sessionExpiredHandler = undefined;
    }
  };
}

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
  sessionExpiredMode?: "handle" | "throw";
  skipAuthRefresh?: boolean;
};

type InternalApiRequestOptions = ApiRequestOptions & {
  hasRetriedAfterRefresh?: boolean;
};

export async function apiRequest<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  return apiRequestInternal(path, options);
}

async function apiRequestInternal<T>(
  path: string,
  {
    fallback = "Request failed",
    headers,
    body,
    sessionExpiredMode = "handle",
    skipAuthRefresh = false,
    hasRetriedAfterRefresh = false,
    ...init
  }: InternalApiRequestOptions = {},
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
    const errorMessage = await getErrorResponse(response, fallback);

    if (
      shouldRefreshSession(path, response, errorMessage, skipAuthRefresh, hasRetriedAfterRefresh)
    ) {
      try {
        await getRefreshSessionPromise();
      } catch (error) {
        if (sessionExpiredMode === "handle") {
          await handleRefreshFailure();
        }
        throw error;
      }

      try {
        return await apiRequestInternal<T>(path, {
          fallback,
          headers,
          body,
          sessionExpiredMode,
          skipAuthRefresh,
          hasRetriedAfterRefresh: true,
          ...init,
        });
      } catch (error) {
        if (error instanceof Error && isRefreshableAuthError(error.message)) {
          if (sessionExpiredMode === "handle") {
            await handleRefreshFailure();
          }
        }

        throw error;
      }
    }

    throw new Error(errorMessage);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function shouldRefreshSession(
  path: string,
  response: Response,
  errorMessage: string,
  skipAuthRefresh: boolean,
  hasRetriedAfterRefresh: boolean,
) {
  return (
    response.status === 401 &&
    isRefreshableAuthError(errorMessage) &&
    !skipAuthRefresh &&
    !hasRetriedAfterRefresh &&
    path !== REFRESH_SESSION_PATH
  );
}

function isRefreshableAuthError(errorMessage: string) {
  return errorMessage === EXPIRED_SESSION_MESSAGE || errorMessage === MISSING_CREDENTIALS_MESSAGE;
}

function getRefreshSessionPromise() {
  refreshSessionPromise ??= refreshSession().finally(() => {
    refreshSessionPromise = undefined;
  });

  return refreshSessionPromise;
}

async function refreshSession() {
  const response = await fetch(`${API_BASE_URL}${REFRESH_SESSION_PATH}`, {
    method: "POST",
    headers: {
      Accept: "application/json",
    },
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error(await getErrorResponse(response, "Session expired"));
  }
}

async function handleRefreshFailure() {
  if (sessionExpiredHandler) {
    await sessionExpiredHandler();
    return;
  }

  if (typeof window !== "undefined") {
    window.location.assign("/sign-in");
  }
}
