import { afterEach, describe, expect, it, vi } from "vitest";
import { apiRequest } from "./browser";

afterEach(() => vi.unstubAllGlobals());

describe("browser API", () => {
  it("refreshes an expired cookie session once and retries the request", async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(Response.json({ error: "Session expired" }, { status: 401 }))
      .mockResolvedValueOnce(Response.json({ message: "refreshed" }))
      .mockResolvedValueOnce(Response.json({ name: "John Doe" }));
    vi.stubGlobal("fetch", fetchMock);

    await expect(apiRequest<{ name: string }>("/auth/me")).resolves.toEqual({ name: "John Doe" });
    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(fetchMock.mock.calls[1]?.[0]).toBe("/api/auth/refresh");
  });

  it("does not loop when refresh is rejected", async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(Response.json({ error: "Session expired" }, { status: 401 }))
      .mockResolvedValueOnce(Response.json({ error: "Refresh expired" }, { status: 401 }));
    vi.stubGlobal("fetch", fetchMock);

    await expect(apiRequest("/auth/me")).rejects.toMatchObject({ status: 401 });
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});
