import { createFileRoute } from "@tanstack/react-router";
import PeopleTablePage from "#/components/people/people-table";

export const Route = createFileRoute("/_main/people/staff/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PeopleTablePage
      title="Staff"
      description="Review internal users across administrative, support, and operations roles."
      category="staff"
    />
  );
}
