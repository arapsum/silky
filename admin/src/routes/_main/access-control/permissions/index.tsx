import { createFileRoute } from "@tanstack/react-router";
import PermissionsPage from "#/components/access-control/permissions";

export const Route = createFileRoute("/_main/access-control/permissions/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <PermissionsPage />;
}
