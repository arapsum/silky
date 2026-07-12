import { createFileRoute } from "@tanstack/react-router";

import OrderDetailPage from "#/components/orders/order-detail";

export const Route = createFileRoute("/_main/orders/$pid/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <OrderDetailPage pid={pid} />;
}
