import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  return (
    <div>
      <section>
        <h1>Dashboard</h1>
      </section>
    </div>
  );
}
