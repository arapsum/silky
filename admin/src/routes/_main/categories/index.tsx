import { createFileRoute } from "@tanstack/react-router";
import CategoryCatalogue from "#/components/categories/category-catalogue";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/categories/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.categories.read]),
  component: RouteComponent,
});

function RouteComponent() {
  return <CategoryCatalogue />;
}
