import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Navbar } from "#/components/navbar";
import { AppSidebar } from "#/components/sidebar/app-sidebar";
import { SidebarInset, SidebarProvider } from "#/components/ui/sidebar";

export const Route = createFileRoute("/_main")({
  component: MainLayout,
});

function MainLayout() {
  return (
    <SidebarProvider>
      <div className="relative flex min-h-dvh w-full">
        <AppSidebar />
        <SidebarInset className="flex min-w-0 flex-col">
          <Navbar />
          <div className="mx-auto w-full max-w-360 px-4 py-4 md:px-8">
            <Outlet />
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}
