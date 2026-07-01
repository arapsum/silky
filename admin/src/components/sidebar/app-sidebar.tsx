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
    title: "Home",
    icon: <HouseIcon className="size-4" />,
    link: "#",
  },
  {
    id: "products",
    title: "Products",
    icon: <PackageIcon className="size-4" />,
    link: "#",
    subs: [
      {
        title: "Catalogue",
        link: "#",
        icon: <PackageIcon className="size-4" />,
      },
      {
        title: "Add",
        link: "#",
        icon: <LinkIcon className="size-4" />,
      },
      {
        title: "Discounts",
        link: "#",
        icon: <PercentIcon className="size-4" />,
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
        title: "Catalogue",
        link: "#",
        icon: <ChartPieIcon className="size-4" />,
      },
      {
        title: "Add",
        link: "#",
        icon: <PulseIcon className="size-4" />,
      },
    ],
  },
  {
    id: "benefits",
    title: "Benefits",
    icon: <SparkleIcon className="size-4" />,
    link: "#",
  },
  {
    id: "customers",
    title: "Customers",
    icon: <UsersIcon className="size-4" />,
    link: "#",
  },
  {
    id: "sales",
    title: "Sales",
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
    ],
  },
  {
    id: "storefront",
    title: "Store front",
    icon: <StorefrontIcon className="size-4" />,
    link: "#",
  },
  {
    id: "analytics",
    title: "Analytics",
    icon: <ChartLineUpIcon className="size-4" />,
    link: "#",
  },
  {
    id: "finance",
    title: "Finance",
    icon: <CurrencyDollarIcon className="size-4" />,
    link: "#",
    subs: [
      { title: "Incoming", link: "#" },
      { title: "Outgoing", link: "#" },
      { title: "Payout Account", link: "#" },
    ],
  },
  {
    id: "settings",
    title: "Settings",
    icon: <GearIcon className="size-4" />,
    link: "#",
    subs: [
      { title: "General", link: "#" },
      { title: "Webhooks", link: "#" },
      { title: "Custom Fields", link: "#" },
    ],
  },
];

export function DashboardSidebar() {
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
          {!isCollapsed && <span className="font-semibold text-black dark:text-white">Acme</span>}
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
