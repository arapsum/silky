import type React from "react";

import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

type ErrorStateProps = {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  retryLabel?: string;
  onRetry?: () => void;
  action?: React.ReactNode;
  className?: string;
};

export function ErrorState({
  icon,
  title,
  description,
  retryLabel = "Retry",
  onRetry,
  action,
  className,
}: ErrorStateProps) {
  return (
    <div
      className={cn(
        "flex min-h-64 flex-col items-center justify-center border px-6 text-center",
        className,
      )}
    >
      {icon && <div className="mb-3 text-muted-foreground">{icon}</div>}
      <h2 className="text-base font-semibold">{title}</h2>
      {description && <p className="mt-1 max-w-sm text-sm text-muted-foreground">{description}</p>}
      {action ?? (
        <>
          {onRetry && (
            <Button type="button" variant="outline" className="mt-4" onClick={onRetry}>
              {retryLabel}
            </Button>
          )}
        </>
      )}
    </div>
  );
}
