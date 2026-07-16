import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AccountOverview } from "./AccountOverview";

const api = vi.hoisted(() => ({
  loadSession: vi.fn(),
  listOrders: vi.fn(),
}));

vi.mock("@/lib/api/browser", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/api/browser")>();
  return {
    ...original,
    orderApi: { ...original.orderApi, list: api.listOrders },
    sessionApi: { ...original.sessionApi, me: api.loadSession },
  };
});

beforeEach(() => {
  api.loadSession.mockResolvedValue({
    pid: "f4636b97-3594-40e1-b914-79453feacfd2",
    email: "eredin@example.com",
    name: "Eredin Breac Glas",
    image: null,
    verified: false,
    roles: [],
    permissions: [],
    createdAt: "2026-07-16T09:00:00Z",
    updatedAt: "2026-07-16T09:00:00Z",
  });
  api.listOrders.mockResolvedValue({ data: [], pagination: {} });
});

afterEach(cleanup);

describe("AccountOverview", () => {
  it("validates a profile email while the customer is editing it", async () => {
    const user = userEvent.setup();
    render(<AccountOverview />);

    const email = await screen.findByLabelText("Email address");
    await user.clear(email);
    await user.type(email, "eredin:mail.com");

    expect(await screen.findByText("Enter a valid email address.")).toBeTruthy();
    expect(email.getAttribute("aria-invalid")).toBe("true");
  });
});
