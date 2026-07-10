"use client";

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import { Link, useRouterState } from "@tanstack/react-router";
import { CaretDownIcon, CaretUpIcon } from "@phosphor-icons/react";
import type React from "react";
import { useEffect, useState } from "react";

export type Route = {
  id: string;
  title: string;
  icon?: React.ReactNode;
  link: string;
  subs?: {
    title: string;
    link: string;
    icon?: React.ReactNode;
  }[];
};

export default function DashboardNavigation({ routes }: { routes: Route[] }) {
  const { state } = useSidebar();
  const isCollapsed = state === "collapsed";
  const pathname = useRouterState({ select: (routerState) => routerState.location.pathname });
  const activeGroup = routes.find((route) =>
    route.subs?.some((subRoute) => isRouteActive(pathname, subRoute.link)),
  );
  const [openCollapsible, setOpenCollapsible] = useState<string | null>(activeGroup?.id ?? null);

  useEffect(() => {
    if (activeGroup) setOpenCollapsible(activeGroup.id);
  }, [activeGroup]);

  return (
    <SidebarMenu>
      {routes.map((route) => {
        const isOpen = !isCollapsed && openCollapsible === route.id;
        const hasSubRoutes = !!route.subs?.length;
        const isActive = isRouteActive(pathname, route.link);
        const hasActiveChild = route.subs?.some((subRoute) =>
          isRouteActive(pathname, subRoute.link),
        );

        return (
          <SidebarMenuItem key={route.id}>
            {hasSubRoutes ? (
              <Collapsible
                open={isOpen}
                onOpenChange={(open) => setOpenCollapsible(open ? route.id : null)}
                className="w-full"
              >
                <CollapsibleTrigger
                  render={
                    <SidebarMenuButton
                      isActive={hasActiveChild}
                      className={cn(
                        "flex w-full items-center rounded-none border-l-2 border-transparent px-2 transition-colors data-active:border-primary data-active:bg-primary/10 data-active:text-primary",
                        isOpen
                          ? "bg-sidebar-accent text-foreground"
                          : "text-muted-foreground hover:bg-sidebar-accent hover:text-foreground",
                        isCollapsed && "justify-center",
                      )}
                    />
                  }
                >
                  {route.icon}
                  {!isCollapsed && (
                    <span className="ml-2 flex-1 text-sm font-medium">{route.title}</span>
                  )}
                  {!isCollapsed && hasSubRoutes && (
                    <span className="ml-auto">
                      {isOpen ? (
                        <CaretUpIcon className="size-4" />
                      ) : (
                        <CaretDownIcon className="size-4" />
                      )}
                    </span>
                  )}
                </CollapsibleTrigger>

                {!isCollapsed && (
                  <CollapsibleContent>
                    <SidebarMenuSub className="my-1 ml-3.5">
                      {route.subs?.map((subRoute) => (
                        <SidebarMenuSubItem key={`${route.id}-${subRoute.title}`}>
                          {subRoute.link === "#" ? (
                            <span
                              className="flex h-8 min-w-0 items-center gap-2 px-3 text-sm text-muted-foreground/50"
                              aria-disabled="true"
                            >
                              {subRoute.icon}
                              <span className="truncate">{subRoute.title}</span>
                              <span className="ml-auto text-[10px] font-medium">Soon</span>
                            </span>
                          ) : (
                            <SidebarMenuSubButton
                              isActive={isRouteActive(pathname, subRoute.link)}
                              className="h-8 rounded-none border-l-2 border-transparent data-active:border-primary data-active:bg-primary/10 data-active:text-primary"
                              render={<Link to={subRoute.link} preload="intent" />}
                            >
                              {subRoute.icon}
                              <span>{subRoute.title}</span>
                            </SidebarMenuSubButton>
                          )}
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                )}
              </Collapsible>
            ) : (
              <SidebarMenuButton
                tooltip={route.title}
                isActive={isActive}
                render={<Link to={route.link} preload="intent" />}
                className={cn(
                  "flex items-center rounded-none border-l-2 border-transparent px-2 text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-foreground data-active:border-primary data-active:bg-primary/10 data-active:text-primary",
                  isCollapsed && "justify-center",
                )}
              >
                {route.icon}
                {!isCollapsed && <span className="ml-2 text-sm font-medium">{route.title}</span>}
              </SidebarMenuButton>
            )}
          </SidebarMenuItem>
        );
      })}
    </SidebarMenu>
  );
}

function isRouteActive(pathname: string, link: string) {
  if (link === "#") return false;
  if (link === "/") return pathname === "/";
  return pathname === link || pathname.startsWith(`${link}/`);
}
