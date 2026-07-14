import { createFileRoute } from "@tanstack/react-router";

import ProductDetailPage from "#/components/products/product-detail";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/products/$pid/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.products.read]),
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <ProductDetailPage pid={pid} />;
}
