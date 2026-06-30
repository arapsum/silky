import { createFileRoute } from "@tanstack/react-router";

import { LoginForm } from "#/components/login-form.tsx";

export const Route = createFileRoute("/_auth/sign-in/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <main className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
      <div className="w-full max-w-sm">
        <LoginForm />
      </div>
    </main>
  );
}
