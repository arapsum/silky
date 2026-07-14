import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { LockKeyIcon, SignOutIcon } from "@phosphor-icons/react";
import { toast } from "sonner";
import { currentUserQueryKey } from "#/api/account.ts";
import { logout } from "#/api/auth.ts";
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
    <main className="grid min-h-dvh place-items-center bg-muted/30 px-6 py-12">
      <section className="w-full max-w-lg rounded-xl border bg-background p-8 shadow-sm">
        <span className="flex size-11 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <LockKeyIcon className="size-5" weight="bold" />
        </span>
        <p className="mt-6 text-sm font-medium text-primary">Access restricted</p>
        <h1 className="mt-2 text-2xl font-semibold tracking-tight">You cannot open this page</h1>
        <p className="mt-3 text-sm leading-6 text-muted-foreground">
          Your account is authenticated, but its assigned roles do not grant access to this area of
          Silk Admin. Contact an administrator if you believe your access should be updated.
        </p>
        <Button
          type="button"
          variant="outline"
          className="mt-7 rounded-lg"
          disabled={logoutMutation.isPending}
          onClick={() => logoutMutation.mutate()}
        >
          <SignOutIcon className="size-4" />
          {logoutMutation.isPending ? "Signing out..." : "Sign out"}
        </Button>
      </section>
    </main>
  );
}
