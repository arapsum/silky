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
import { initials } from "@/utils/formatters";
import { cn } from "@/lib/utils";
import { PERMISSIONS } from "@/lib/access";
import { useAccess } from "@/hooks/use-access";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
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
import DashboardNavigation, { filterRoutesByAccess } from "./nav-main";

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
        link: "/products",
        icon: <PackageIcon className="size-4" />,
        permissions: [PERMISSIONS.products.read],
      },
      {
        title: "Variants & SKUs",
        link: "#",
        icon: <LinkIcon className="size-4" />,
        permissions: [PERMISSIONS.products.read],
      },
      {
        title: "Attributes",
        link: "#",
        icon: <SparkleIcon className="size-4" />,
        permissions: [PERMISSIONS.products.read],
      },
      {
        title: "Product Options",
        link: "#",
        icon: <ChartPieIcon className="size-4" />,
        permissions: [PERMISSIONS.products.read],
      },
      {
        title: "Media Library",
        link: "#",
        icon: <StorefrontIcon className="size-4" />,
        permissions: [PERMISSIONS.media.read],
      },
    ],
  },
  {
    id: "categories",
    title: "Categories",
    icon: <ChartPieIcon className="size-4" />,
    link: "#",
    permissions: [PERMISSIONS.categories.read],
    subs: [
      {
        title: "Category List",
        link: "/categories",
        icon: <ChartPieIcon className="size-4" />,
        permissions: [PERMISSIONS.categories.read],
      },
      {
        title: "Create Category",
        link: "/categories/create",
        icon: <StorefrontIcon className="size-4" />,
        permissions: [PERMISSIONS.categories.create],
      },
      {
        title: "Category Tree",
        link: "#",
        icon: <PulseIcon className="size-4" />,
        permissions: [PERMISSIONS.categories.read],
      },
    ],
  },
  {
    id: "people",
    title: "People",
    icon: <UsersIcon className="size-4" />,
    link: "#",
    permissions: [PERMISSIONS.users.read],
    subs: [
      {
        title: "Customers",
        link: "/people/customers",
        icon: <UsersIcon className="size-4" />,
        permissions: [PERMISSIONS.users.read],
      },
      {
        title: "Staff",
        link: "/people/staff",
        icon: <GearIcon className="size-4" />,
        permissions: [PERMISSIONS.users.read],
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
        permissions: [PERMISSIONS.roles.read],
      },
      {
        title: "Permissions",
        link: "/access-control/permissions",
        icon: <LinkIcon className="size-4" />,
        permissions: [PERMISSIONS.roles.read, PERMISSIONS.permissions.read],
      },
    ],
  },
  {
    id: "orders",
    title: "Orders",
    icon: <ShoppingBagIcon className="size-4" />,
    link: "#",
    permissions: [PERMISSIONS.orders.read],
    subs: [
      {
        title: "Orders",
        link: "/orders",
        icon: <ShoppingBagIcon className="size-4" />,
        permissions: [PERMISSIONS.orders.read],
      },
      {
        title: "Subscriptions",
        link: "#",
        icon: <InfinityIcon className="size-4" />,
        permissions: [PERMISSIONS.orders.read],
      },
      {
        title: "Fulfillment",
        link: "#",
        icon: <PackageIcon className="size-4" />,
        permissions: [PERMISSIONS.orders.read],
      },
      {
        title: "Returns",
        link: "#",
        icon: <LinkIcon className="size-4" />,
        permissions: [PERMISSIONS.orders.read],
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

export function AppSidebar() {
  const { state } = useSidebar();
  const { currentUser } = useAccess();
  const isCollapsed = state === "collapsed";
  const accessibleRoutes = filterRoutesByAccess(dashboardRoutes, currentUser);

  return (
    <Sidebar variant="sidebar" collapsible="icon">
      <SidebarHeader
        className={cn(
          "flex min-h-16 justify-center border-b px-3",
          isCollapsed
            ? "flex-row items-center md:flex-col md:items-center"
            : "flex-row items-center justify-between",
        )}
      >
        <Link to="/" className="flex items-center gap-2.5" aria-label="Silk dashboard">
          <Logo className="h-8 w-7 text-foreground" />
          {!isCollapsed && (
            <span className="text-base font-semibold tracking-tight text-foreground">Silk</span>
          )}
        </Link>
      </SidebarHeader>
      <SidebarContent className="gap-0 px-2 py-3">
        <DashboardNavigation routes={accessibleRoutes} />
      </SidebarContent>
      <SidebarFooter className="border-t px-2 py-2">
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
    retry: false,
  });

  const currentUser = currentUserQuery.data;
  const fallback = initials(currentUser?.name);

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
                "h-12 w-full justify-start gap-3 rounded-lg px-2",
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

        <DropdownMenuContent align="end" side="right" sideOffset={8} className="w-64 rounded-lg">
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

          <DropdownMenuItem
            variant="destructive"
            className="rounded-lg"
            onClick={() => setIsLogoutDialogOpen(true)}
          >
            <SignOutIcon className="size-4" />
            Sign out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialogContent className="rounded-lg">
        <AlertDialogHeader>
          <AlertDialogTitle>Sign out?</AlertDialogTitle>
          <AlertDialogDescription>
            {currentUser
              ? `You are signed in as ${currentUser.email}. You will be redirected to sign in after signing out.`
              : "You will be redirected to sign in after signing out."}
          </AlertDialogDescription>
        </AlertDialogHeader>

        <AlertDialogFooter>
          <AlertDialogCancel className="rounded-lg" disabled={logoutMutation.isPending}>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            className="rounded-lg"
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
