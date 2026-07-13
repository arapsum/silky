import { Badge } from "#/components/ui/badge";
import { titleCase } from "#/utils/formatters";

export const ORDER_STATUSES = [
  "pending",
  "confirmed",
  "processing",
  "completed",
  "cancelled",
  "refunded",
] as const;

export const PAYMENT_STATUSES = ["pending", "authorized", "paid", "failed", "refunded"] as const;

export const FULFILLMENT_STATUSES = ["unfulfilled", "partial", "fulfilled", "cancelled"] as const;

export function orderStatusClass(status: string) {
  switch (status.toLowerCase()) {
    case "completed":
    case "paid":
    case "fulfilled":
      return "border-primary/30 bg-primary/10 text-primary";
    case "cancelled":
    case "failed":
    case "refunded":
      return "border-destructive/30 bg-destructive/10 text-destructive";
    case "pending":
    case "confirmed":
    case "processing":
    case "authorized":
    case "partial":
    case "unfulfilled":
      return "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300";
    default:
      return "border-border bg-muted text-muted-foreground";
  }
}

export function OrderStatusBadge({ value }: { value: string }) {
  return (
    <Badge variant="outline" className={orderStatusClass(value)}>
      <span className="size-1 rounded-full bg-current" aria-hidden />
      {titleCase(value)}
    </Badge>
  );
}
