import { createFileRoute } from "@tanstack/react-router";

import CategoryDetailPage from "#/components/categories/category-detail";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/categories/$pid/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.categories.read]),
  component: RouteComponent,
});

function RouteComponent() {
  const { pid } = Route.useParams();

  return <CategoryDetailPage pid={pid} />;
}
