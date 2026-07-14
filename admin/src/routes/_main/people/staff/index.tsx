import { createFileRoute } from "@tanstack/react-router";
import PeopleTablePage from "#/components/people/people-table";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/people/staff/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.users.read]),
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PeopleTablePage
      title="Staff"
      description="Review internal users across administrative, support, and operations roles."
      category="staff"
    />
  );
}
