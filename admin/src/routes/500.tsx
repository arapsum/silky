import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowLeftIcon } from "@phosphor-icons/react";

import { StatusPage } from "#/components/status-page.tsx";
import { Button } from "#/components/ui/button.tsx";

export const Route = createFileRoute("/500")({
  component: ServerErrorPage,
});

function ServerErrorPage() {
  return (
    <StatusPage
      kind="error"
      title="This page is not working"
      description="We are fixing the problem. Please try again later."
      actions={
        <Button render={<Link to="/" />}>
          <ArrowLeftIcon data-icon="inline-start" />
          Go to dashboard
        </Button>
      }
    />
  );
}
