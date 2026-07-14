import type React from "react";
import {
  BarricadeIcon,
  FileMagnifyingGlassIcon,
  ShieldWarningIcon,
  TrafficConeIcon,
  WarningDiamondIcon,
} from "@phosphor-icons/react";

import { cn } from "#/lib/utils";

const statusPageConfig = {
  unauthorized: {
    code: "401",
    title: "Authentication required",
    description: "Sign in with an authorised staff account to continue to Silk Admin.",
    icon: ShieldWarningIcon,
  },
  forbidden: {
    code: "403",
    title: "Access forbidden",
    description: "Your account does not have permission to open this area of Silk Admin.",
    icon: BarricadeIcon,
  },
  notFound: {
    code: "404",
    title: "Page not found",
    description: "The page may have moved, or the address may be incorrect.",
    icon: FileMagnifyingGlassIcon,
  },
  error: {
    code: "500",
    title: "Something went wrong",
    description:
      "Silk Admin could not complete this request. Try again or return to the dashboard.",
    icon: WarningDiamondIcon,
  },
  maintenance: {
    code: null,
    title: "System maintenance",
    description: "Silk Admin is temporarily unavailable while maintenance is in progress.",
    icon: TrafficConeIcon,
  },
} as const;

export type StatusPageKind = keyof typeof statusPageConfig;

type StatusPageProps = {
  kind: StatusPageKind;
  title?: string;
  description?: string;
  actions?: React.ReactNode;
  className?: string;
};

export function StatusPage({ kind, title, description, actions, className }: StatusPageProps) {
  const config = statusPageConfig[kind];
  const Icon = config.icon;

  return (
    <main
      className={cn(
        "grid min-h-dvh place-items-center bg-muted/40 px-4 py-8 sm:px-6 sm:py-12",
        className,
      )}
    >
      <section
        className="flex w-full max-w-xl flex-col items-center rounded-xl border bg-card px-6 py-10 text-center text-card-foreground shadow-sm sm:px-12 sm:py-14"
        aria-labelledby="status-page-title"
        aria-describedby="status-page-description"
      >
        <div
          className="relative mb-8 grid h-40 w-full max-w-64 place-items-center overflow-hidden rounded-xl border bg-muted/35"
          aria-hidden="true"
        >
          <div className="relative grid size-24 place-items-center rounded-xl border bg-background shadow-sm">
            <Icon className="size-12 text-primary" weight="duotone" />
          </div>
        </div>

        {config.code && (
          <p className="text-4xl font-semibold tracking-tight text-primary">{config.code}</p>
        )}
        <h1 id="status-page-title" className="mt-3 text-2xl font-semibold tracking-tight">
          {title ?? config.title}
        </h1>
        <p
          id="status-page-description"
          className="mt-3 max-w-md text-sm leading-6 text-muted-foreground"
        >
          {description ?? config.description}
        </p>
        {actions && <div className="mt-7 flex flex-wrap justify-center gap-3">{actions}</div>}
      </section>
    </main>
  );
}
