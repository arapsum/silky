"use client";

import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
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
  CalendarIcon,
  EnvelopeIcon,
  GearIcon,
  LinkIcon,
  MagnifyingGlassIcon,
  MoonIcon,
  QuestionIcon,
  RocketLaunchIcon,
  SunIcon,
  TrendUpIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useEffect, useState } from "react";

const notifications = [
  {
    id: "mark-bush",
    avatar: "",
    fallback: "MB",
    title: "Mark Bush",
    time: "12 Minutes ago",
    meta: "New post",
    unread: true,
  },
  {
    id: "aaron-black",
    avatar: "",
    fallback: "AB",
    title: "Aaron Black",
    time: "27 Minutes ago",
    meta: "New comment",
    unread: true,
  },
  {
    id: "anna-campaign",
    avatar: "",
    fallback: "AN",
    title: "Anna has applied to create an ad for your campaign",
    time: "2 hours ago",
    meta: "New request for campaign",
    actions: true,
  },
  {
    id: "jason-file",
    avatar: "",
    fallback: "JS",
    title: "Jason attached the file",
    time: "6 hours ago",
    meta: "Attached files",
    attachment: "Work examples.com",
  },
];

const commandSuggestions = [
  {
    id: "mail",
    title: "Mail - App",
    icon: EnvelopeIcon,
  },
  {
    id: "contact",
    title: "Contact - App",
    icon: CalendarIcon,
  },
  {
    id: "sales",
    title: "Sales - Dashboard",
    icon: TrendUpIcon,
  },
  {
    id: "pricing",
    title: "Pricing - Page",
    icon: RocketLaunchIcon,
  },
  {
    id: "faq",
    title: "FAQ - Page",
    icon: QuestionIcon,
  },
];

export function DashboardNavbar() {
  const [isCommandOpen, setIsCommandOpen] = useState(false);

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
      <header className="sticky top-0 z-20  bg-background/95 px-4 py-3 backdrop-blur md:px-8">
        <div className="mx-auto flex h-12 w-full max-w-360 items-center gap-3 rounded-xl border bg-card px-4 shadow-sm">
          <SidebarTrigger className="size-8" />
          <Separator orientation="vertical" className="h-8 my-auto" />

          <button
            type="button"
            className="relative flex h-9 min-w-0 flex-1 items-center rounded-lg px-9 text-left text-sm text-muted-foreground outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring md:max-w-sm"
            onClick={() => setIsCommandOpen(true)}
          >
            <MagnifyingGlassIcon
              className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
              aria-hidden
            />
            <span className="truncate">Type to search...</span>
            <kbd className="absolute right-2 top-1/2 hidden h-5 -translate-y-1/2 items-center rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground sm:flex">
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
  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title="Command search"
      description="Search for a page or command"
      className="max-w-[calc(100%-2rem)] rounded-xl! sm:max-w-md"
    >
      <Command>
        <CommandInput placeholder="Type a command or search..." autoFocus />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            {commandSuggestions.map((suggestion) => (
              <CommandItem key={suggestion.id} onSelect={() => onOpenChange(false)}>
                <suggestion.icon className="size-4" />
                <span>{suggestion.title}</span>
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
            className="relative"
            aria-label="Notifications"
          />
        }
      >
        <BellIcon className="size-4" />
        <span className="absolute right-2 top-2 size-1.5 rounded-full bg-destructive" />
      </PopoverTrigger>

      <PopoverContent
        align="end"
        sideOffset={10}
        className="w-[min(calc(100vw-2rem),28rem)] gap-0 rounded-md p-0"
      >
        <div className="flex items-center justify-between px-3 py-3">
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            Notifications
          </p>
          <span className="rounded-full bg-blue-50 px-2 py-1 text-xs font-medium text-blue-600 dark:bg-blue-950 dark:text-blue-300">
            8 New
          </span>
        </div>

        <div className="flex items-end justify-between border-b px-3">
          <div className="flex items-center gap-5 text-sm">
            <button type="button" className="border-b-2 border-foreground pb-2 text-foreground">
              Inbox
            </button>
            <button type="button" className="pb-2 text-muted-foreground hover:text-foreground">
              General
            </button>
          </div>
          <Button type="button" variant="ghost" size="icon-sm" aria-label="Notification settings">
            <GearIcon className="size-4" />
          </Button>
        </div>

        <div className="grid">
          {notifications.map((notification) => (
            <div key={notification.id} className="border-b px-3 py-4 last:border-b-0">
              <div className="flex items-start gap-3">
                <Avatar className="mt-0.5">
                  <AvatarImage src={notification.avatar} alt={notification.title} />
                  <AvatarFallback>{notification.fallback}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                  <div className="flex items-start justify-between gap-3">
                    <p className="text-sm font-medium leading-snug">{notification.title}</p>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-xs"
                      className="-mr-1 -mt-1 shrink-0"
                      aria-label="Dismiss notification"
                    >
                      <XIcon className="size-3.5" />
                    </Button>
                  </div>

                  <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <span>{notification.time}</span>
                    <span className="size-1 rounded-full bg-blue-200" />
                    <span>{notification.meta}</span>
                    {notification.unread && (
                      <span className="ml-auto size-1.5 rounded-full bg-blue-500" />
                    )}
                  </div>

                  {notification.actions && (
                    <div className="mt-3 flex items-center gap-2">
                      <Button
                        type="button"
                        variant="secondary"
                        size="sm"
                        className="h-8 rounded-md px-3"
                      >
                        Decline
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        className="h-8 rounded-md bg-blue-600 px-3 text-white hover:bg-blue-700"
                      >
                        Accept
                      </Button>
                    </div>
                  )}

                  {notification.attachment && (
                    <a
                      href="#"
                      className="mt-3 flex items-center gap-2 text-sm text-foreground hover:underline"
                    >
                      <LinkIcon className="size-4" />
                      {notification.attachment}
                    </a>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}

function ThemeToggle() {
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    setIsDark(document.documentElement.classList.contains("dark"));
  }, []);

  const toggleTheme = () => {
    setIsDark((current) => {
      const next = !current;
      document.documentElement.classList.toggle("dark", next);
      return next;
    });
  };

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      aria-label={isDark ? "Switch to light theme" : "Switch to dark theme"}
      onClick={toggleTheme}
    >
      {isDark ? <SunIcon className="size-4" /> : <MoonIcon className="size-4" />}
    </Button>
  );
}
