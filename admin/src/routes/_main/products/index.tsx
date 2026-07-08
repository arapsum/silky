import { createFileRoute } from "@tanstack/react-router";

import ProductCatalogue from "#/components/products/product-catalogue";

export const Route = createFileRoute("/_main/products/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <ProductCatalogue />;
}
