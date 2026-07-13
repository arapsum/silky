/** Formats a dashed, underscored, or space-separated identifier for display. */
export function titleCase(value: string) {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

/** Builds a compact two-character fallback for avatar content. */
export function initials(name?: string, fallback = "U") {
  const value = (name ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");

  return value || fallback;
}

/** Formats catalogue counts using the active locale. */
export function formatNumber(value: number) {
  return Intl.NumberFormat().format(value);
}

/** Formats a numeric API amount as currency using the active locale. */
export function formatCurrency(value: number | string, currency = "USD") {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
  }).format(Number(value));
}

const shortDateFormatter = new Intl.DateTimeFormat(undefined, {
  day: "2-digit",
  month: "short",
  year: "numeric",
});

/** Formats a date without a time for catalogue and table rows. */
export function formatDate(value: string) {
  return shortDateFormatter.format(new Date(value));
}

/** Formats a timestamp, returning a useful fallback when it is absent. */
export function formatDateTime(value: string | null, fallback = "Not available") {
  if (!value) return fallback;

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
