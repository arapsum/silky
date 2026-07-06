import { createFileRoute } from "@tanstack/react-router";
import CategoryCatalogue from "#/components/categories/category-catalogue";

export const Route = createFileRoute("/_main/categories/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <CategoryCatalogue />;
}
