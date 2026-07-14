import { createFileRoute } from "@tanstack/react-router";

import EditProductForm from "#/components/products/edit-product-form";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/products/$pid/edit/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.products.update]),
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <EditProductForm pid={pid} />;
}
