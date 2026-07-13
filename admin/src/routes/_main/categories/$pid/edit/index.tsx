import { createFileRoute } from "@tanstack/react-router";

import EditCategoryForm from "#/components/categories/edit-category-form";

export const Route = createFileRoute("/_main/categories/$pid/edit/")({
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <EditCategoryForm pid={pid} />;
}
