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
import { cn } from "@/lib/utils";
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
import { Logo } from "./logo";
import type { Route } from "./nav-main";
import DashboardNavigation from "./nav-main";

const currentUser = {
  name: "Silk Admin",
  email: "admin@silk.local",
  image: "",
  fallback: "SA",
};

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
        link: "#",
        icon: <UsersIcon className="size-4" />,
      },
      {
        title: "Administrators",
        link: "#",
        icon: <GearIcon className="size-4" />,
      },
      {
        title: "Profiles",
        link: "#",
        icon: <LinkIcon className="size-4" />,
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
        link: "#",
        icon: <GearIcon className="size-4" />,
      },
      {
        title: "Permissions",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
      {
        title: "Role Assignments",
        link: "#",
        icon: <UsersIcon className="size-4" />,
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
  const [isLogoutDialogOpen, setIsLogoutDialogOpen] = useState(false);

  const handleConfirmLogout = () => {
    setIsLogoutDialogOpen(false);
  };

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
          <Avatar>
            <AvatarImage src={currentUser.image} alt={currentUser.name} />
            <AvatarFallback>{currentUser.fallback}</AvatarFallback>
          </Avatar>
          {!isCollapsed && (
            <span className="grid min-w-0 flex-1 text-left">
              <span className="truncate text-sm font-medium">{currentUser.name}</span>
              <span className="truncate text-xs text-muted-foreground">{currentUser.email}</span>
            </span>
          )}
        </DropdownMenuTrigger>

        <DropdownMenuContent align="end" side="right" sideOffset={8} className="w-64">
          <DropdownMenuLabel>
            <div className="flex items-center gap-3">
              <Avatar size="lg">
                <AvatarImage src={currentUser.image} alt={currentUser.name} />
                <AvatarFallback>{currentUser.fallback}</AvatarFallback>
              </Avatar>
              <div className="grid min-w-0">
                <span className="truncate text-sm font-medium text-foreground">
                  {currentUser.name}
                </span>
                <span className="truncate text-xs font-normal text-muted-foreground">
                  {currentUser.email}
                </span>
              </div>
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
            You will need to sign in again before continuing in the admin dashboard.
          </AlertDialogDescription>
        </AlertDialogHeader>

        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={handleConfirmLogout}>
            Sign out
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
