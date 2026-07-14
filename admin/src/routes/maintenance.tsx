import { createFileRoute } from "@tanstack/react-router";

import { StatusPage } from "#/components/status-page.tsx";

export const Route = createFileRoute("/maintenance")({
  component: MaintenancePage,
});

function MaintenancePage() {
  return (
    <StatusPage
      kind="maintenance"
      description="We will be back as soon as the scheduled maintenance is complete."
    />
  );
}
