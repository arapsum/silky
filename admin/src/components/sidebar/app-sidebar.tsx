"use client";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { currentUserQueryKey, getCurrentUser, type CurrentUser } from "@/api/account";
import { logout } from "@/api/auth";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  useSidebar,
} from "@/components/ui/sidebar";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import {
  PulseIcon,
  CurrencyDollarIcon,
  HouseIcon,
  InfinityIcon,
  LinkIcon,
  PackageIcon,
  PercentIcon,
  ChartPieIcon,
  GearIcon,
  ShoppingBagIcon,
  SparkleIcon,
  StorefrontIcon,
  ChartLineUpIcon,
  UsersIcon,
  SignOutIcon,
} from "@phosphor-icons/react";
import { useState } from "react";
import { toast } from "sonner";
import { Logo } from "./logo";
import type { Route } from "./nav-main";
import DashboardNavigation from "./nav-main";

const dashboardRoutes: Route[] = [
  {
    id: "home",
    title: "Dashboard",
    icon: <HouseIcon className="size-4" />,
    link: "/",
  },
  {
    id: "catalogue",
    title: "Catalogue",
    icon: <PackageIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Products",
        link: "#",
        icon: <PackageIcon className="size-4" />,
      },
      {
        title: "Variants & SKUs",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
      {
        title: "Attributes",
        link: "#",
        icon: <SparkleIcon className="size-4" />,
      },
      {
        title: "Product Options",
        link: "#",
        icon: <ChartPieIcon className="size-4" />,
      },
      {
        title: "Media Library",
        link: "#",
        icon: <StorefrontIcon className="size-4" />,
      },
    ],
  },
  {
    id: "categories",
    title: "Categories",
    icon: <ChartPieIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Category List",
        link: "#",
        icon: <ChartPieIcon className="size-4" />,
      },
      {
        title: "Category Tree",
        link: "#",
        icon: <PulseIcon className="size-4" />,
      },
    ],
  },
  {
    id: "people",
    title: "People",
    icon: <UsersIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Customers",
        link: "/people/customers",
        icon: <UsersIcon className="size-4" />,
      },
      {
        title: "Staff",
        link: "/people/staff",
        icon: <GearIcon className="size-4" />,
      },
    ],
  },
  {
    id: "access-control",
    title: "Access Control",
    icon: <GearIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Roles",
        link: "/access-control/roles",
        icon: <GearIcon className="size-4" />,
      },
      {
        title: "Permissions",
        link: "/access-control/permissions",
        icon: <LinkIcon className="size-4" />,
      },
    ],
  },
  {
    id: "orders",
    title: "Orders",
    icon: <ShoppingBagIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Orders",
        link: "#",
        icon: <ShoppingBagIcon className="size-4" />,
      },
      {
        title: "Subscriptions",
        link: "#",
        icon: <InfinityIcon className="size-4" />,
      },
      {
        title: "Fulfillment",
        link: "#",
        icon: <PackageIcon className="size-4" />,
      },
      {
        title: "Returns",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
    ],
  },
  {
    id: "marketing",
    title: "Marketing",
    icon: <SparkleIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Benefits",
        link: "#",
        icon: <SparkleIcon className="size-4" />,
      },
      {
        title: "Discounts",
        link: "#",
        icon: <PercentIcon className="size-4" />,
      },
      {
        title: "Campaigns",
        link: "#",
        icon: <PulseIcon className="size-4" />,
      },
    ],
  },
  {
    id: "storefront",
    title: "Storefront",
    icon: <StorefrontIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Pages",
        link: "#",
        icon: <StorefrontIcon className="size-4" />,
      },
      {
        title: "Navigation",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
    ],
  },
  {
    id: "analytics",
    title: "Analytics",
    icon: <ChartLineUpIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Overview",
        link: "#",
        icon: <ChartLineUpIcon className="size-4" />,
      },
      {
        title: "Catalogue Health",
        link: "#",
        icon: <ChartPieIcon className="size-4" />,
      },
      {
        title: "Inventory",
        link: "#",
        icon: <PackageIcon className="size-4" />,
      },
    ],
  },
  {
    id: "finance",
    title: "Finance",
    icon: <CurrencyDollarIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Payments",
        link: "#",
        icon: <CurrencyDollarIcon className="size-4" />,
      },
      {
        title: "Refunds",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
      {
        title: "Payout Accounts",
        link: "#",
        icon: <GearIcon className="size-4" />,
      },
    ],
  },
  {
    id: "settings",
    title: "Settings",
    icon: <GearIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Account",
        link: "/settings",
        icon: <GearIcon className="size-4" />,
      },
      {
        title: "Webhooks",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
      {
        title: "Custom Fields",
        link: "#",
        icon: <SparkleIcon className="size-4" />,
      },
      {
        title: "Seed Data",
        link: "#",
        icon: <PackageIcon className="size-4" />,
      },
    ],
  },
];

function userInitials(name?: string) {
  const initials = (name ?? "User")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");

  return initials || "U";
}

export function AppSidebar() {
  const { state } = useSidebar();
  const isCollapsed = state === "collapsed";

  return (
    <Sidebar variant="inset" collapsible="icon">
      <SidebarHeader
        className={cn(
          "flex md:pt-3.5",
          isCollapsed
            ? "flex-row items-center justify-between gap-y-4 md:flex-col md:items-start md:justify-start"
            : "flex-row items-center justify-between",
        )}
      >
        <a href="#" className="flex items-center gap-2">
          <Logo className="h-8 w-8" />
          {!isCollapsed && <span className="font-semibold text-black dark:text-white">Silk</span>}
        </a>
      </SidebarHeader>
      <SidebarContent className="gap-4 px-2 py-4">
        <DashboardNavigation routes={dashboardRoutes} />
      </SidebarContent>
      <SidebarFooter className="px-2">
        <UserAccountMenu isCollapsed={isCollapsed} />
      </SidebarFooter>
    </Sidebar>
  );
}

function UserAccountMenu({ isCollapsed }: { isCollapsed: boolean }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [isLogoutDialogOpen, setIsLogoutDialogOpen] = useState(false);
  const currentUserQuery = useQuery({
    queryKey: currentUserQueryKey,
    queryFn: getCurrentUser,
  });

  const currentUser = currentUserQuery.data;
  const fallback = userInitials(currentUser?.name);

  const logoutMutation = useMutation({
    mutationFn: logout,
    onSuccess: async (response) => {
      setIsLogoutDialogOpen(false);
      queryClient.removeQueries({ queryKey: currentUserQueryKey });
      toast.success(response.message || "Signed out successfully", {
        id: "sign-out-success",
      });
      await navigate({ to: "/sign-in" });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "sign-out-error",
      });
    },
  });

  async function handleConfirmLogout() {
    await logoutMutation.mutateAsync();
  }

  return (
    <AlertDialog open={isLogoutDialogOpen} onOpenChange={setIsLogoutDialogOpen}>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              type="button"
              variant="ghost"
              className={cn(
                "h-12 w-full justify-start gap-3 rounded-xl px-2",
                isCollapsed && "size-10 justify-center px-0",
              )}
            />
          }
        >
          <AccountAvatar user={currentUser} fallback={fallback} />
          {!isCollapsed && (
            <AccountSummary
              user={currentUser}
              isLoading={currentUserQuery.isLoading}
              hasError={currentUserQuery.isError}
            />
          )}
        </DropdownMenuTrigger>

        <DropdownMenuContent align="end" side="right" sideOffset={8} className="w-64">
          <DropdownMenuLabel>
            <div className="flex items-center gap-3">
              <AccountAvatar user={currentUser} fallback={fallback} size="lg" />
              <AccountSummary
                user={currentUser}
                isLoading={currentUserQuery.isLoading}
                hasError={currentUserQuery.isError}
                className="min-w-0"
              />
            </div>
          </DropdownMenuLabel>

          <DropdownMenuSeparator />

          <DropdownMenuItem variant="destructive" onClick={() => setIsLogoutDialogOpen(true)}>
            <SignOutIcon className="size-4" />
            Sign out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Sign out?</AlertDialogTitle>
          <AlertDialogDescription>
            {currentUser
              ? `You are signed in as ${currentUser.email}. You will be redirected to sign in after signing out.`
              : "You will be redirected to sign in after signing out."}
          </AlertDialogDescription>
        </AlertDialogHeader>

        <AlertDialogFooter>
          <AlertDialogCancel disabled={logoutMutation.isPending}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            disabled={logoutMutation.isPending}
            onClick={handleConfirmLogout}
          >
            {logoutMutation.isPending ? "Signing out..." : "Sign out"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

function AccountAvatar({
  user,
  fallback,
  size,
}: {
  user?: CurrentUser;
  fallback: string;
  size?: "default" | "lg";
}) {
  return (
    <Avatar size={size}>
      <AvatarImage src={user?.image ?? undefined} alt={user?.name ?? "Current user"} />
      <AvatarFallback>{fallback}</AvatarFallback>
    </Avatar>
  );
}

function AccountSummary({
  user,
  isLoading,
  hasError,
  className,
}: {
  user?: CurrentUser;
  isLoading: boolean;
  hasError: boolean;
  className?: string;
}) {
  if (isLoading) {
    return (
      <span className={cn("grid min-w-0 flex-1 gap-1", className)}>
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-3 w-32" />
      </span>
    );
  }

  if (hasError || !user) {
    return (
      <span className={cn("grid min-w-0 flex-1 text-left", className)}>
        <span className="truncate text-sm font-medium">Account unavailable</span>
        <span className="truncate text-xs text-muted-foreground">Sign out and try again</span>
      </span>
    );
  }

  return (
    <span className={cn("grid min-w-0 flex-1 text-left", className)}>
      <span className="truncate text-sm font-medium">{user.name}</span>
      <span className="truncate text-xs text-muted-foreground">{user.email}</span>
    </span>
  );
}
