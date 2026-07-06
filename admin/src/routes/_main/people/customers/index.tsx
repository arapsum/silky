import { createFileRoute } from "@tanstack/react-router";
import PeopleTablePage from "#/components/people/people-table";

export const Route = createFileRoute("/_main/people/customers/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PeopleTablePage
      title="Customers"
      description="Review customer accounts and their access status."
      category="customers"
    />
  );
}
