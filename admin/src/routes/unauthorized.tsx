import { createFileRoute, Link } from "@tanstack/react-router";
import { SignInIcon } from "@phosphor-icons/react";

import { StatusPage } from "#/components/status-page.tsx";
import { Button } from "#/components/ui/button.tsx";

export const Route = createFileRoute("/unauthorized")({
  component: UnauthorizedPage,
});

function UnauthorizedPage() {
  return (
    <StatusPage
      kind="unauthorized"
      actions={
        <Button render={<Link to="/sign-in" />}>
          <SignInIcon data-icon="inline-start" />
          Sign in
        </Button>
      }
    />
  );
}
