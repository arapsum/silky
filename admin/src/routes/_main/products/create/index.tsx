import { createFileRoute } from "@tanstack/react-router";

import CreateProductForm from "#/components/products/create-product-form";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/products/create/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.products.create]),
  component: RouteComponent,
});

function RouteComponent() {
  return <CreateProductForm />;
}
