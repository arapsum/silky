import { createFileRoute } from "@tanstack/react-router";

import { OrdersTable } from "#/components/orders/orders-table";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/orders/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.orders.read]),
  component: OrdersTable,
});
