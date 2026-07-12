import { createFileRoute } from "@tanstack/react-router";

import { OrdersTable } from "#/components/orders/orders-table";

export const Route = createFileRoute("/_main/orders/")({
  component: OrdersTable,
});
