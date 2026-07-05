import { createFileRoute } from "@tanstack/react-router";
import PeopleTablePage from "#/components/people/people-table";

export const Route = createFileRoute("/_main/people/administrators/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PeopleTablePage
      title="Administrators"
      description="Review administrator accounts that can manage the system."
      role="administrator"
    />
  );
}
