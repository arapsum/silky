import {
  HeadContent,
  Link,
  Scripts,
  createRootRouteWithContext,
  type ErrorComponentProps,
} from "@tanstack/react-router";
import { ArrowClockwiseIcon, ArrowLeftIcon } from "@phosphor-icons/react";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { TanStackDevtools } from "@tanstack/react-devtools";

import TanStackQueryDevtools from "../integrations/tanstack-query/devtools";
import { Toaster } from "#/components/ui/sonner.tsx";

import appCss from "../styles.css?url";

import type { QueryClient } from "@tanstack/react-query";
import { ThemeProvider } from "#/components/theme-provider";
import { TooltipProvider } from "#/components/ui/tooltip";
import { StatusPage } from "#/components/status-page.tsx";
import { Button } from "#/components/ui/button.tsx";

interface MyRouterContext {
  queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        title: "Silk Admin",
      },
    ],
    links: [
      {
        rel: "stylesheet",
        href: appCss,
      },
    ],
  }),
  errorComponent: RootErrorPage,
  notFoundComponent: NotFoundPage,
  shellComponent: RootDocument,
});

function NotFoundPage() {
  return (
    <StatusPage
      kind="notFound"
      actions={
        <Button render={<Link to="/" />}>
          <ArrowLeftIcon data-icon="inline-start" />
          Go to dashboard
        </Button>
      }
    />
  );
}

function RootErrorPage({ reset }: ErrorComponentProps) {
  return (
    <StatusPage
      kind="error"
      actions={
        <>
          <Button type="button" onClick={reset}>
            <ArrowClockwiseIcon data-icon="inline-start" />
            Try again
          </Button>
          <Button variant="outline" render={<Link to="/" />}>
            Go to dashboard
          </Button>
        </>
      }
    />
  );
}

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body>
        <ThemeProvider>
          <TooltipProvider>{children}</TooltipProvider>
          <Toaster richColors position="top-right" duration={5000} />
        </ThemeProvider>
        <TanStackDevtools
          config={{
            position: "bottom-right",
          }}
          plugins={[
            {
              name: "Tanstack Router",
              render: <TanStackRouterDevtoolsPanel />,
            },
            TanStackQueryDevtools,
          ]}
        />
        <Scripts />
      </body>
    </html>
  );
}
