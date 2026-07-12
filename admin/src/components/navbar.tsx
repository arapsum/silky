"use client";

import { Button } from "#/components/ui/button";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "#/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import { Separator } from "#/components/ui/separator";
import { SidebarTrigger } from "#/components/ui/sidebar";
import {
  BellIcon,
  ChartPieIcon,
  GearIcon,
  HouseIcon,
  KeyIcon,
  MagnifyingGlassIcon,
  MoonIcon,
  PackageIcon,
  ShieldCheckIcon,
  SunIcon,
  UsersIcon,
} from "@phosphor-icons/react";
import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";

const commandSuggestions = [
  {
    id: "dashboard",
    title: "Dashboard",
    description: "Admin overview",
    to: "/",
    icon: HouseIcon,
  },
  {
    id: "products",
    title: "Products",
    description: "Manage catalogue products",
    to: "/products",
    icon: PackageIcon,
  },
  {
    id: "categories",
    title: "Categories",
    description: "Organise the catalogue",
    to: "/categories",
    icon: ChartPieIcon,
  },
  {
    id: "customers",
    title: "Customers",
    description: "Review customer accounts",
    to: "/people/customers",
    icon: UsersIcon,
  },
  {
    id: "roles",
    title: "Roles",
    description: "Manage access roles",
    to: "/access-control/roles",
    icon: ShieldCheckIcon,
  },
  {
    id: "permissions",
    title: "Permissions",
    description: "Review role permissions",
    to: "/access-control/permissions",
    icon: KeyIcon,
  },
  {
    id: "settings",
    title: "Settings",
    description: "Manage your account",
    to: "/settings",
    icon: GearIcon,
  },
] as const;

function sectionLabel(pathname: string) {
  if (pathname.startsWith("/products")) return "Catalogue";
  if (pathname.startsWith("/categories")) return "Categories";
  if (pathname.startsWith("/people")) return "People";
  if (pathname.startsWith("/access-control")) return "Access Control";
  if (pathname.startsWith("/settings")) return "Settings";
  return "Dashboard";
}

export function Navbar() {
  const [isCommandOpen, setIsCommandOpen] = useState(false);
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setIsCommandOpen((open) => !open);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <>
      <header className="sticky top-0 z-20 border-b bg-background/95 backdrop-blur">
        <div className="mx-auto flex h-16 w-full max-w-360 items-center gap-3 px-4 md:px-8">
          <SidebarTrigger className="size-8 rounded-lg" />
          <Separator orientation="vertical" className="my-auto h-7" />
          <span className="hidden min-w-28 text-sm font-semibold md:block">
            {sectionLabel(pathname)}
          </span>

          <button
            type="button"
            className="relative flex h-9 min-w-0 flex-1 items-center rounded-lg border bg-muted/30 px-9 text-left text-sm text-muted-foreground outline-none transition-colors hover:bg-muted focus-visible:border-primary focus-visible:ring-3 focus-visible:ring-primary/15 md:max-w-md"
            onClick={() => setIsCommandOpen(true)}
          >
            <MagnifyingGlassIcon
              className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
              aria-hidden
            />
            <span className="truncate">Type to search...</span>
            <kbd className="absolute right-2 top-1/2 hidden h-5 -translate-y-1/2 items-center rounded-md border bg-background px-1.5 font-mono text-[10px] font-medium text-muted-foreground sm:flex">
              Ctrl K
            </kbd>
          </button>

          <div className="ml-auto flex items-center gap-1">
            <NotificationsButton />
            <ThemeToggle />
          </div>
        </div>
      </header>

      <CommandSearchDialog open={isCommandOpen} onOpenChange={setIsCommandOpen} />
    </>
  );
}

function CommandSearchDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const navigate = useNavigate();

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title="Command search"
      description="Search for a page or command"
      className="max-w-[calc(100%-2rem)] rounded-lg! [&_[data-slot=command]]:rounded-lg! [&_[data-slot=input-group]]:rounded-lg! sm:max-w-md"
    >
      <Command>
        <CommandInput placeholder="Type a command or search..." autoFocus />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            {commandSuggestions.map((suggestion) => (
              <CommandItem
                key={suggestion.id}
                className="rounded-lg!"
                onSelect={() => {
                  onOpenChange(false);
                  void navigate({ to: suggestion.to });
                }}
              >
                <suggestion.icon className="size-4" />
                <div className="min-w-0">
                  <p className="text-sm font-medium">{suggestion.title}</p>
                  <p className="text-xs text-muted-foreground">{suggestion.description}</p>
                </div>
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}

function NotificationsButton() {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="relative rounded-lg"
            aria-label="Notifications"
          />
        }
      >
        <BellIcon className="size-4" />
      </PopoverTrigger>

      <PopoverContent
        align="end"
        sideOffset={10}
        className="w-[min(calc(100vw-2rem),22rem)] gap-0 rounded-lg p-0"
      >
        <div className="border-b px-4 py-3">
          <p className="text-sm font-semibold">Notifications</p>
          <p className="mt-0.5 text-xs text-muted-foreground">Store and account updates</p>
        </div>
        <div className="flex flex-col items-center px-6 py-9 text-center">
          <span className="flex size-10 items-center justify-center bg-primary/10 text-primary">
            <BellIcon className="size-5" />
          </span>
          <p className="mt-3 text-sm font-medium">No notifications</p>
          <p className="mt-1 max-w-56 text-xs leading-5 text-muted-foreground">
            Store and account updates will appear here when they are available.
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function ThemeToggle() {
  const [isMounted, setIsMounted] = useState(false);
  const { resolvedTheme, setTheme } = useTheme();

  useEffect(() => {
    setIsMounted(true);
  }, []);

  const isDark = isMounted && resolvedTheme === "dark";

  const toggleTheme = () => {
    setTheme(isDark ? "light" : "dark");
  };

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      className="rounded-lg"
      aria-label={isDark ? "Switch to light theme" : "Switch to dark theme"}
      onClick={toggleTheme}
    >
      {isDark ? <SunIcon className="size-4" /> : <MoonIcon className="size-4" />}
    </Button>
  );
}
