import { createFileRoute } from "@tanstack/react-router";

import OrderDetailPage from "#/components/orders/order-detail";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/orders/$pid/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.orders.read]),
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <OrderDetailPage pid={pid} />;
}
