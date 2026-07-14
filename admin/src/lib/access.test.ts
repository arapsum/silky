import { describe, expect, it } from "vitest";
import {
  hasAnyPermission,
  hasEveryPermission,
  hasPermission,
  PERMISSIONS,
  type AccessSubject,
} from "./access";

const manager: AccessSubject = {
  roles: ["manager"],
  permissions: [PERMISSIONS.admin.access, PERMISSIONS.products.read, PERMISSIONS.products.update],
};

describe("access helpers", () => {
  it("checks a single effective permission", () => {
    expect(hasPermission(manager, PERMISSIONS.products.read)).toBe(true);
    expect(hasPermission(manager, PERMISSIONS.roles.update)).toBe(false);
  });

  it("requires every permission when requested", () => {
    expect(
      hasEveryPermission(manager, [PERMISSIONS.products.read, PERMISSIONS.products.update]),
    ).toBe(true);
    expect(
      hasEveryPermission(manager, [PERMISSIONS.products.read, PERMISSIONS.products.delete]),
    ).toBe(false);
  });

  it("accepts any matching permission", () => {
    expect(hasAnyPermission(manager, [PERMISSIONS.roles.update, PERMISSIONS.products.update])).toBe(
      true,
    );
    expect(hasAnyPermission(undefined, [PERMISSIONS.products.read])).toBe(false);
  });
});
