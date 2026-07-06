import { createFileRoute } from "@tanstack/react-router";
import CreateCategoryForm from "#/components/categories/create-category-form";

export const Route = createFileRoute("/_main/categories/create/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <CreateCategoryForm />;
}
