import { createFileRoute } from "@tanstack/react-router";

import CategoryDetailPage from "#/components/categories/category-detail";

export const Route = createFileRoute("/_main/categories/$pid/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <CategoryDetailPage pid={pid} />;
}
