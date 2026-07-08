import { createFileRoute } from "@tanstack/react-router";

import ProductDetailPage from "#/components/products/product-detail";

export const Route = createFileRoute("/_main/products/$pid/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <ProductDetailPage pid={pid} />;
}
