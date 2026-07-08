import { createFileRoute } from "@tanstack/react-router";

import CreateProductForm from "#/components/products/create-product-form";

export const Route = createFileRoute("/_main/products/create/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <CreateProductForm />;
}
