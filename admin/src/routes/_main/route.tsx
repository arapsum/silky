import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Outlet } from "@tanstack/react-router";
import { toast } from "sonner";
import { setSessionExpiredHandler } from "#/api/client.ts";
import { currentUserQueryKey } from "#/api/account.ts";
import { Navbar } from "#/components/navbar";
import { AppSidebar } from "#/components/sidebar/app-sidebar";
import { SidebarInset, SidebarProvider } from "#/components/ui/sidebar";

export const Route = createFileRoute("/_main")({
  component: MainLayout,
});

function MainLayout() {
  const queryClient = useQueryClient();
  const navigate = Route.useNavigate();

  useEffect(() => {
    return setSessionExpiredHandler(async () => {
      queryClient.removeQueries({ queryKey: currentUserQueryKey });
      toast.error("Your session has expired. Please sign in again.", {
        id: "session-expired",
      });
      await navigate({ to: "/sign-in" });
    });
  }, [navigate, queryClient]);

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
