import { createFileRoute } from "@tanstack/react-router";
import CreateCategoryForm from "#/components/categories/create-category-form";
import { PERMISSIONS } from "#/lib/access.ts";
import { requirePermissions } from "#/lib/route-access.ts";

export const Route = createFileRoute("/_main/categories/create/")({
  beforeLoad: ({ context }) =>
    requirePermissions(context.currentUser, [PERMISSIONS.categories.create]),
  component: RouteComponent,
});

function RouteComponent() {
  return <CreateCategoryForm />;
}
