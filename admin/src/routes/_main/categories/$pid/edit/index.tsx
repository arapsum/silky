import { createFileRoute } from "@tanstack/react-router";

import EditCategoryForm from "#/components/categories/edit-category-form";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/categories/$pid/edit/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.categories.update]),
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <EditCategoryForm pid={pid} />;
}
