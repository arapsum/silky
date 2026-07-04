import { Navbar } from "#/components/navbar";
import { AppSidebar } from "#/components/sidebar/app-sidebar";
import { SidebarInset, SidebarProvider } from "#/components/ui/sidebar";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  return (
    <SidebarProvider>
      <div className="relative flex h-dvh w-full">
        <AppSidebar />
        <SidebarInset className="flex flex-col">
          <Navbar />
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}
