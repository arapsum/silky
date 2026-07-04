import { createFileRoute } from "@tanstack/react-router";
import AccountSettingsPage from "#/components/settings";

export const Route = createFileRoute("/_main/settings/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <AccountSettingsPage />;
}
