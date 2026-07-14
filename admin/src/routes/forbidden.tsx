import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeftIcon, SignOutIcon } from "@phosphor-icons/react";
import { toast } from "sonner";
import { currentUserQueryKey } from "#/api/account.ts";
import { logout } from "#/api/auth.ts";
import { StatusPage } from "#/components/status-page.tsx";
import { Button } from "#/components/ui/button.tsx";

export const Route = createFileRoute("/forbidden")({
  component: ForbiddenPage,
});

function ForbiddenPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const logoutMutation = useMutation({
    mutationFn: logout,
    onSuccess: async () => {
      queryClient.removeQueries({ queryKey: currentUserQueryKey });
      await navigate({ to: "/sign-in" });
    },
    onError: (error) => toast.error(error.message, { id: "forbidden-sign-out-error" }),
  });

  return (
    <StatusPage
      kind="forbidden"
      description="Your account is authenticated, but its assigned roles do not grant access to this area. Contact an administrator if your access should be updated."
      actions={
        <>
          <Button render={<Link to="/" />}>
            <ArrowLeftIcon data-icon="inline-start" />
            Go to dashboard
          </Button>
          <Button
            type="button"
            variant="outline"
            disabled={logoutMutation.isPending}
            onClick={() => logoutMutation.mutate()}
          >
            <SignOutIcon data-icon="inline-start" />
            {logoutMutation.isPending ? "Signing out..." : "Sign out"}
          </Button>
        </>
      }
    />
  );
}
