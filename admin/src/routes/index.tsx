import { DashboardNavbar } from "#/components/dashboard-navbar";
import { DashboardSidebar } from "#/components/sidebar/app-sidebar";
import { SidebarInset, SidebarProvider } from "#/components/ui/sidebar";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  return (
    <SidebarProvider>
      <div className="relative flex h-dvh w-full">
        <DashboardSidebar />
        <SidebarInset className="flex flex-col">
          <DashboardNavbar />
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}
