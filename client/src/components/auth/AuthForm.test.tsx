import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AuthForm } from "./AuthForm";

const login = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api/browser", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/api/browser")>();
  return {
    ...original,
    sessionApi: { ...original.sessionApi, login },
  };
});

afterEach(cleanup);

describe("AuthForm", () => {
  it("uses a POST fallback so credentials cannot enter the URL", () => {
    const { container } = render(<AuthForm mode="login" />);
    const form = container.querySelector("form");

    expect(form?.getAttribute("action")).toBe("/auth/login");
    expect(form?.getAttribute("method")).toBe("post");
  });

  it("validates input with the Zod resolver before calling the login API", async () => {
    const user = userEvent.setup();
    render(<AuthForm mode="login" />);

    await user.type(screen.getByLabelText("Email address"), "not-an-email");
    await user.type(screen.getByLabelText("Password"), "short");
    await user.click(screen.getByRole("button", { name: "Sign in" }));

    expect(await screen.findByText("Enter a valid email address.")).toBeTruthy();
    expect(await screen.findByText("Use at least 8 characters.")).toBeTruthy();
    expect(login).not.toHaveBeenCalled();
  });
});
