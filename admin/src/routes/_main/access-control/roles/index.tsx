import { createFileRoute } from "@tanstack/react-router";
import RolesPage from "#/components/access-control/roles";

export const Route = createFileRoute("/_main/access-control/roles/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <RolesPage />;
}
