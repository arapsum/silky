import { createFileRoute } from "@tanstack/react-router";

import EditProductForm from "#/components/products/edit-product-form";

export const Route = createFileRoute("/_main/products/$pid/edit/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <EditProductForm pid={pid} />;
}
