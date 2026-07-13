import { ArrowClockwiseIcon } from "@phosphor-icons/react";

import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

type RefreshButtonProps = {
  onRefresh: () => void;
  isRefreshing?: boolean;
  disabled?: boolean;
  className?: string;
};

/** A consistent refresh action for data-backed admin pages. */
export function RefreshButton({
  onRefresh,
  isRefreshing = false,
  disabled = false,
  className,
}: RefreshButtonProps) {
  return (
    <Button
      type="button"
      variant="outline"
      className={cn("rounded-lg", className)}
      onClick={onRefresh}
      disabled={disabled || isRefreshing}
    >
      <ArrowClockwiseIcon className={cn("size-4", isRefreshing && "animate-spin")} />
      Refresh
    </Button>
  );
}
