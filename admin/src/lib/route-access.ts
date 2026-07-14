import { redirect } from "@tanstack/react-router";
import type { CurrentUser } from "#/api/account.ts";
import {
  hasEveryPermission,
  hasPermission,
  PERMISSIONS,
  type PermissionName,
} from "#/lib/access.ts";

export function requireAdminAccess(user: CurrentUser) {
  if (!hasPermission(user, PERMISSIONS.admin.access)) {
    throw redirect({ to: "/forbidden" });
  }
}

export function requirePermissions(user: CurrentUser, permissions: readonly PermissionName[]) {
  if (!hasEveryPermission(user, permissions)) {
    throw redirect({ to: "/forbidden" });
  }
}
