import { createFileRoute } from "@tanstack/react-router";
import RoleAssignmentsPage from "#/components/access-control/role-assignments";

export const Route = createFileRoute("/_main/access-control/role-assignments/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <RoleAssignmentsPage />;
}
