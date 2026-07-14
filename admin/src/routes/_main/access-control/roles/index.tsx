import { createFileRoute } from "@tanstack/react-router";
import RolesPage from "#/components/access-control/roles";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/access-control/roles/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.roles.read]),
  component: RouteComponent,
});

function RouteComponent() {
  return <RolesPage />;
}
