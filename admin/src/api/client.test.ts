import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { API_BASE_URL, apiRequest, setSessionExpiredHandler } from "./client.ts";

function jsonResponse(status: number, body: unknown) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
    },
  });
}

function expiredSessionResponse() {
  return jsonResponse(401, { error: "Expired session", code: "session_expired" });
}

function missingCredentialsResponse() {
  return jsonResponse(401, { error: "Missing credentials", code: "missing_credentials" });
}

describe("apiRequest", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    fetchMock.mockReset();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    setSessionExpiredHandler(undefined);
  });

  it("refreshes an expired session and retries the original request", async () => {
    fetchMock
      .mockResolvedValueOnce(expiredSessionResponse())
      .mockResolvedValueOnce(jsonResponse(200, { message: "Session refreshed" }))
      .mockResolvedValueOnce(jsonResponse(200, { name: "Silk Admin" }));

    await expect(apiRequest("/auth/me")).resolves.toEqual({ name: "Silk Admin" });

    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      `${API_BASE_URL}/auth/refresh`,
      expect.objectContaining({
        credentials: "include",
        method: "POST",
      }),
    );
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it("refreshes when the access cookie is missing but the refresh cookie may still exist", async () => {
    fetchMock
      .mockResolvedValueOnce(missingCredentialsResponse())
      .mockResolvedValueOnce(jsonResponse(200, { message: "Session refreshed" }))
      .mockResolvedValueOnce(jsonResponse(200, { name: "Silk Admin" }));

    await expect(apiRequest("/auth/me")).resolves.toEqual({ name: "Silk Admin" });

    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      `${API_BASE_URL}/auth/refresh`,
      expect.objectContaining({
        credentials: "include",
        method: "POST",
      }),
    );
  });

  it("shares the same refresh request across concurrent expired requests", async () => {
    const requestAttempts = new Map<string, number>();

    fetchMock.mockImplementation((input) => {
      const url = String(input);

      if (url.endsWith("/auth/refresh")) {
        return Promise.resolve(jsonResponse(200, { message: "Session refreshed" }));
      }

      const attempts = requestAttempts.get(url) ?? 0;
      requestAttempts.set(url, attempts + 1);

      if (attempts === 0) {
        return Promise.resolve(expiredSessionResponse());
      }

      return Promise.resolve(jsonResponse(200, { path: url.replace(API_BASE_URL, "") }));
    });

    const firstRequest = apiRequest("/users");
    const secondRequest = apiRequest("/roles");

    await expect(Promise.all([firstRequest, secondRequest])).resolves.toEqual([
      { path: "/users" },
      { path: "/roles" },
    ]);
    expect(
      fetchMock.mock.calls.filter(([input]) => String(input).endsWith("/auth/refresh")),
    ).toHaveLength(1);
  });

  it("notifies the app when refresh fails", async () => {
    const sessionExpiredHandler = vi.fn();
    setSessionExpiredHandler(sessionExpiredHandler);

    fetchMock
      .mockResolvedValueOnce(expiredSessionResponse())
      .mockResolvedValueOnce(jsonResponse(401, { error: "Invalid token" }));

    await expect(apiRequest("/users")).rejects.toThrow("Invalid token");
    expect(sessionExpiredHandler).toHaveBeenCalledTimes(1);
  });

  it("lets callers handle session expiry without notifying the app", async () => {
    const sessionExpiredHandler = vi.fn();
    setSessionExpiredHandler(sessionExpiredHandler);

    fetchMock
      .mockResolvedValueOnce(expiredSessionResponse())
      .mockResolvedValueOnce(jsonResponse(401, { error: "Invalid token" }));

    await expect(apiRequest("/users", { sessionExpiredMode: "throw" })).rejects.toThrow(
      "Invalid token",
    );
    expect(sessionExpiredHandler).not.toHaveBeenCalled();
  });

  it("does not notify the app when a retried request fails for another reason", async () => {
    const sessionExpiredHandler = vi.fn();
    setSessionExpiredHandler(sessionExpiredHandler);

    fetchMock
      .mockResolvedValueOnce(expiredSessionResponse())
      .mockResolvedValueOnce(jsonResponse(200, { message: "Session refreshed" }))
      .mockResolvedValueOnce(jsonResponse(403, { error: "Missing permission" }));

    await expect(apiRequest("/users")).rejects.toThrow("Missing permission");
    expect(sessionExpiredHandler).not.toHaveBeenCalled();
  });

  it("notifies the app when a refreshed request still has no credentials", async () => {
    const sessionExpiredHandler = vi.fn();
    setSessionExpiredHandler(sessionExpiredHandler);

    fetchMock
      .mockResolvedValueOnce(expiredSessionResponse())
      .mockResolvedValueOnce(jsonResponse(200, { message: "Session refreshed" }))
      .mockResolvedValueOnce(missingCredentialsResponse());

    await expect(apiRequest("/users")).rejects.toThrow("Missing credentials");
    expect(sessionExpiredHandler).toHaveBeenCalledTimes(1);
  });

  it("does not refresh non-recoverable authentication failures", async () => {
    fetchMock.mockResolvedValueOnce(jsonResponse(401, { error: "Invalid token" }));

    await expect(apiRequest("/users")).rejects.toThrow("Invalid token");
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("can opt out of session refresh", async () => {
    fetchMock.mockResolvedValueOnce(expiredSessionResponse());

    await expect(apiRequest("/auth/logout", { skipAuthRefresh: true })).rejects.toThrow(
      "Expired session",
    );
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});
