import { act, cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { toast } from "@/lib/toast";
import { ToastViewport } from "./ToastViewport";

afterEach(() => {
  cleanup();
  window.sessionStorage.clear();
});

describe("ToastViewport", () => {
  it("announces and dismisses transient feedback", async () => {
    const user = userEvent.setup();
    render(<ToastViewport />);

    act(() => toast.success("Profile saved", "Your details are up to date."));

    expect((await screen.findByRole("status")).textContent).toContain("Profile saved");
    await user.click(screen.getByRole("button", { name: "Dismiss notification" }));
    expect(screen.queryByText("Profile saved")).toBeNull();
  });

  it("shows feedback queued before a redirect", async () => {
    toast.flash.success("Welcome back", "You are signed in to Silk.");

    render(<ToastViewport />);

    expect((await screen.findByRole("status")).textContent).toContain("Welcome back");
    expect(window.sessionStorage.length).toBe(0);
  });
});
