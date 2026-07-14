import { createFileRoute } from "@tanstack/react-router";

import ProductCatalogue from "#/components/products/product-catalogue";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/products/")({
  beforeLoad: ({ context }) => requirePermissions(context.currentUser, [PERMISSIONS.products.read]),
  component: RouteComponent,
});

function RouteComponent() {
  return <ProductCatalogue />;
}
