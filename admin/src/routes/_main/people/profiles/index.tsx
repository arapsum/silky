import { createFileRoute } from "@tanstack/react-router";
import PeopleTablePage from "#/components/people/people-table";

export const Route = createFileRoute("/_main/people/profiles/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PeopleTablePage title="Profiles" description="Review all user profiles and assigned roles." />
  );
}
