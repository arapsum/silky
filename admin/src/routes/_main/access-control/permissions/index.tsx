import { createFileRoute } from "@tanstack/react-router";
import PermissionsPage from "#/components/access-control/permissions";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/access-control/permissions/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.roles.read, PERMISSIONS.permissions.read]),
  component: RouteComponent,
});

function RouteComponent() {
  return <PermissionsPage />;
}
