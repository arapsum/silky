import { createFileRoute } from "@tanstack/react-router";

import { LoginForm } from "#/components/login-form.tsx";

type SignInSearch = {
  redirect?: string;
};

export const Route = createFileRoute("/_auth/sign-in/")({
  validateSearch: (search: Record<string, unknown>): SignInSearch => {
    const redirect = search.redirect;

    if (typeof redirect !== "string" || !isSafeRedirect(redirect)) {
      return {};
    }

    return { redirect };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { redirect } = Route.useSearch();

  return (
    <main className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
      <div className="w-full max-w-sm">
        <LoginForm redirectTo={redirect} />
      </div>
    </main>
  );
}

function isSafeRedirect(value: string) {
  return value.startsWith("/") && !value.startsWith("//") && !value.startsWith("/sign-in");
}
