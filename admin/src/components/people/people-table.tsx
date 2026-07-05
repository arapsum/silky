"use client";

import type { ColumnDef } from "@tanstack/react-table";
import { ArrowClockwiseIcon, MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";

import { listUsers, usersQueryKey, type User } from "#/api/users.ts";
import { DataTable } from "#/components/data-table";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { cn } from "#/lib/utils";

type PeopleTablePageProps = {
  title: string;
  description: string;
  role?: string;
};

const dateFormatter = new Intl.DateTimeFormat(undefined, {
  day: "2-digit",
  month: "short",
  year: "numeric",
});

function initials(name?: string) {
  const value = (name ?? "User")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");

  return value || "U";
}

function titleCase(value: string) {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatDate(value: string) {
  return dateFormatter.format(new Date(value));
}

function roleBadgeClass(role: string) {
  switch (role.toLowerCase()) {
    case "administrator":
      return "bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950 dark:text-blue-300 dark:ring-blue-900";
    case "customer":
      return "bg-green-50 text-green-700 ring-green-200 dark:bg-green-950 dark:text-green-300 dark:ring-green-900";
    default:
      return "bg-muted text-muted-foreground ring-border";
  }
}

function verificationBadgeClass(verified: boolean) {
  return verified
    ? "bg-green-50 text-green-700 ring-green-200 dark:bg-green-950 dark:text-green-300 dark:ring-green-900"
    : "bg-amber-50 text-amber-700 ring-amber-200 dark:bg-amber-950 dark:text-amber-300 dark:ring-amber-900";
}

function userColumns(): ColumnDef<User>[] {
  return [
    {
      id: "person",
      header: "Person",
      accessorFn: (user) => `${user.name} ${user.email}`,
      cell: ({ row }) => {
        const user = row.original;

        return (
          <div className="flex min-w-0 items-center gap-3">
            <Avatar size="sm">
              <AvatarImage src={user.image ?? undefined} alt={user.name} />
              <AvatarFallback>{initials(user.name)}</AvatarFallback>
            </Avatar>
            <div className="min-w-0">
              <p className="truncate font-medium">{user.name}</p>
              <p className="truncate text-xs text-muted-foreground">{user.email}</p>
            </div>
          </div>
        );
      },
      size: 320,
    },
    {
      id: "roles",
      header: "Roles",
      accessorFn: (user) => user.roles.map((role) => role.name).join(" "),
      cell: ({ row }) => {
        const roles = row.original.roles;

        if (!roles.length) {
          return <span className="text-sm text-muted-foreground">No roles</span>;
        }

        return (
          <div className="flex flex-wrap gap-1.5">
            {roles.map((role) => (
              <span
                key={role.pid}
                className={cn(
                  "rounded-full px-2 py-0.5 text-xs font-medium ring-1",
                  roleBadgeClass(role.name),
                )}
              >
                {titleCase(role.name)}
              </span>
            ))}
          </div>
        );
      },
      size: 260,
    },
    {
      id: "verified",
      header: "Status",
      accessorFn: (user) => (user.verified ? "verified" : "pending"),
      cell: ({ row }) => {
        const verified = row.original.verified;

        return (
          <span
            className={cn(
              "rounded-full px-2 py-0.5 text-xs font-medium ring-1",
              verificationBadgeClass(verified),
            )}
          >
            {verified ? "Verified" : "Pending"}
          </span>
        );
      },
      size: 140,
    },
    {
      id: "createdAt",
      header: "Joined",
      accessorFn: (user) => new Date(user.createdAt).getTime(),
      cell: ({ row }) => (
        <span className="text-sm text-muted-foreground">{formatDate(row.original.createdAt)}</span>
      ),
      size: 160,
    },
    {
      id: "updatedAt",
      header: "Updated",
      accessorFn: (user) => new Date(user.updatedAt).getTime(),
      cell: ({ row }) => (
        <span className="text-sm text-muted-foreground">{formatDate(row.original.updatedAt)}</span>
      ),
      size: 160,
    },
  ];
}

export default function PeopleTablePage({ title, description, role }: PeopleTablePageProps) {
  const [query, setQuery] = useState("");
  const usersQuery = useQuery({
    queryKey: [...usersQueryKey, role ?? "all"],
    queryFn: () => listUsers(role),
  });

  const users = usersQuery.data ?? [];
  const filteredUsers = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return users;

    return users.filter((user) => {
      const roles = user.roles.map((role) => role.name).join(" ");

      return `${user.name} ${user.email} ${roles}`.toLowerCase().includes(needle);
    });
  }, [query, users]);

  const columns = useMemo(() => userColumns(), []);

  return (
    <div className="w-full pb-10">
      <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{description}</p>
        </div>

        <Button
          type="button"
          variant="outline"
          onClick={() => usersQuery.refetch()}
          disabled={usersQuery.isFetching}
        >
          <ArrowClockwiseIcon className={cn("size-4", usersQuery.isFetching && "animate-spin")} />
          Refresh
        </Button>
      </div>

      <div className="mb-4 grid gap-3 border-b pb-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
        <div className="relative">
          <MagnifyingGlassIcon
            className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search people"
            className="pl-9"
          />
        </div>

        <div className="text-sm text-muted-foreground">
          {users.length} {users.length === 1 ? "person" : "people"}
        </div>
      </div>

      <DataTable
        columns={columns}
        data={filteredUsers}
        getRowId={(user) => user.pid}
        isLoading={usersQuery.isLoading}
        isError={usersQuery.isError}
        errorTitle="People could not be loaded"
        onRetry={() => usersQuery.refetch()}
        emptyTitle={query ? "No matching people" : "No people found"}
        emptyDescription={
          query ? "Try a different search term." : "People will appear here after users are added."
        }
      />
    </div>
  );
}
