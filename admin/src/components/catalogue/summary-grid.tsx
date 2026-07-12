import type React from "react";

import { cn } from "#/lib/utils";

export type SummaryMetric = {
  label: string;
  value: React.ReactNode;
};

type SummaryGridProps = {
  items: SummaryMetric[];
  ariaLabel: string;
  className?: string;
};

/**
 * Presents compact operational counts above a catalogue table.
 *
 * The responsive dividers preserve a single visual surface from one to four
 * metrics, so list pages can supply their own counts without recreating the
 * layout and typography.
 */
export function SummaryGrid({ items, ariaLabel, className }: SummaryGridProps) {
  return (
    <section
      className={cn("mb-5 grid rounded-lg border sm:grid-cols-2 xl:grid-cols-4", className)}
      aria-label={ariaLabel}
    >
      {items.map((item) => (
        <div
          key={item.label}
          className="border-b p-4 last:border-b-0 sm:border-r sm:even:border-r-0 sm:last:border-r-0 xl:border-r xl:border-b-0 xl:last:border-r-0"
        >
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {item.label}
          </p>
          <div className="mt-2 text-2xl font-semibold tabular-nums">{item.value}</div>
        </div>
      ))}
    </section>
  );
}
