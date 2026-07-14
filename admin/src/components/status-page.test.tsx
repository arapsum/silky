import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { StatusPage, type StatusPageKind } from "./status-page";

describe("StatusPage", () => {
  it.each<[StatusPageKind, string, string]>([
    ["unauthorized", "401", "Authentication required"],
    ["forbidden", "403", "Access forbidden"],
    ["notFound", "404", "Page not found"],
    ["error", "500", "Something went wrong"],
  ])("renders the %s status variant", (kind, code, title) => {
    const markup = renderToStaticMarkup(<StatusPage kind={kind} />);

    expect(markup).toContain(code);
    expect(markup).toContain(title);
    expect(markup).toContain('aria-labelledby="status-page-title"');
  });

  it("supports page-specific copy and actions", () => {
    const markup = renderToStaticMarkup(
      <StatusPage
        kind="forbidden"
        description="Ask an administrator to update your access."
        actions={<button type="button">Sign out</button>}
      />,
    );

    expect(markup).toContain("Ask an administrator to update your access.");
    expect(markup).toContain("Sign out");
  });

  it("renders maintenance without an error code", () => {
    const markup = renderToStaticMarkup(<StatusPage kind="maintenance" />);

    expect(markup).toContain("System maintenance");
    expect(markup).not.toContain(">500<");
  });
});
