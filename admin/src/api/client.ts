import { env } from "#/env.ts";

export type ErrorResponse = {
  error?: string;
  message?: string;
  code?: string;
};

export const API_BASE_URL = env.VITE_SERVER_URL ?? "http://127.0.0.1:7150/api";
const EXPIRED_SESSION_CODE = "session_expired";
const MISSING_CREDENTIALS_CODE = "missing_credentials";
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
  const body = await readErrorResponse(response);
  return body.error ?? body.message ?? fallback;
}

async function readErrorResponse(response: Response): Promise<ErrorResponse> {
  try {
    return (await response.json()) as ErrorResponse;
  } catch {
    return {};
  }
}

class ApiError extends Error {
  constructor(
    message: string,
    readonly code?: string,
  ) {
    super(message);
    this.name = "ApiError";
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
    const errorResponse = await readErrorResponse(response);
    const errorMessage = errorResponse.error ?? errorResponse.message ?? fallback;

    if (
      shouldRefreshSession(
        path,
        response,
        errorResponse.code,
        skipAuthRefresh,
        hasRetriedAfterRefresh,
      )
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
        if (error instanceof ApiError && isRefreshableAuthError(error.code)) {
          if (sessionExpiredMode === "handle") {
            await handleRefreshFailure();
          }
        }

        throw error;
      }
    }

    throw new ApiError(errorMessage, errorResponse.code);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function shouldRefreshSession(
  path: string,
  response: Response,
  errorCode: string | undefined,
  skipAuthRefresh: boolean,
  hasRetriedAfterRefresh: boolean,
) {
  return (
    response.status === 401 &&
    isRefreshableAuthError(errorCode) &&
    !skipAuthRefresh &&
    !hasRetriedAfterRefresh &&
    path !== REFRESH_SESSION_PATH
  );
}

function isRefreshableAuthError(errorCode: string | undefined) {
  return errorCode === EXPIRED_SESSION_CODE || errorCode === MISSING_CREDENTIALS_CODE;
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
