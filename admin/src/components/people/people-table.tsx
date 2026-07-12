"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  ArrowClockwiseIcon,
  CaretLeftIcon,
  CaretRightIcon,
  MagnifyingGlassIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useMemo, useState, type FormEvent } from "react";

import { listUsers, usersQueryKey, type User } from "#/api/users.ts";
import { SummaryGrid } from "#/components/catalogue/summary-grid";
import { DataTable } from "#/components/data-table";
import { PageHeader } from "#/components/page-header";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { cn } from "#/lib/utils";

type PeopleTablePageProps = {
  title: string;
  description: string;
  category: "customers" | "staff";
};

const PAGE_SIZE = 20;

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
      return "border-primary/30 bg-primary/10 text-primary";
    case "customer":
      return "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
    default:
      return "border-border bg-muted text-muted-foreground";
  }
}

function verificationBadgeClass(verified: boolean) {
  return verified
    ? "border-primary/30 bg-primary/10 text-primary"
    : "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300";
}

function hasCustomerRole(user: User) {
  return user.roles.some((role) => role.name.toLowerCase() === "customer");
}

function hasStaffRole(user: User) {
  return user.roles.some((role) => role.name.toLowerCase() !== "customer");
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
              <Badge key={role.pid} variant="outline" className={roleBadgeClass(role.name)}>
                {titleCase(role.name)}
              </Badge>
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
          <Badge variant="outline" className={verificationBadgeClass(verified)}>
            <span className="size-1 rounded-full bg-current" aria-hidden />
            {verified ? "Verified" : "Pending"}
          </Badge>
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

export default function PeopleTablePage({ title, description, category }: PeopleTablePageProps) {
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const usersQuery = useQuery({
    queryKey: [...usersQueryKey, category],
    queryFn: () => listUsers(),
  });

  const users = useMemo(() => {
    const data = usersQuery.data ?? [];

    if (category === "staff") {
      return data.filter(hasStaffRole);
    }

    return data.filter((user) => hasCustomerRole(user) && !hasStaffRole(user));
  }, [category, usersQuery.data]);

  const filteredUsers = useMemo(() => {
    const needle = search.toLowerCase();
    if (!needle) return users;

    return users.filter((user) => {
      const roles = user.roles.map((role) => role.name).join(" ");

      return `${user.name} ${user.email} ${roles}`.toLowerCase().includes(needle);
    });
  }, [search, users]);

  const columns = useMemo(() => userColumns(), []);
  const totalPages = Math.max(Math.ceil(filteredUsers.length / PAGE_SIZE), 1);
  const paginatedUsers = filteredUsers.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE);
  const activeFilters = Boolean(search);
  const verifiedUsers = users.filter((user) => user.verified).length;
  const pendingUsers = users.length - verifiedUsers;
  const administratorCount = users.filter((user) =>
    user.roles.some((role) => role.name.toLowerCase() === "administrator"),
  ).length;

  function applySearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSearch(searchInput.trim());
    setPage(1);
  }

  function resetFilters() {
    setSearchInput("");
    setSearch("");
    setPage(1);
  }

  return (
    <div className="w-full pb-10">
      <PageHeader
        title={title}
        subtitle={description}
        actions={
          <Button
            type="button"
            variant="outline"
            className="rounded-lg"
            onClick={() => usersQuery.refetch()}
            disabled={usersQuery.isFetching}
          >
            <ArrowClockwiseIcon className={cn("size-4", usersQuery.isFetching && "animate-spin")} />
            Refresh
          </Button>
        }
      />

      <SummaryGrid
        ariaLabel={`${title} summary`}
        items={[
          {
            label: activeFilters
              ? `Matching ${title.toLowerCase()}`
              : `Total ${title.toLowerCase()}`,
            value: filteredUsers.length,
          },
          { label: "On this page", value: paginatedUsers.length },
          { label: "Verified", value: verifiedUsers },
          {
            label: category === "staff" ? "Administrators" : "Pending verification",
            value: category === "staff" ? administratorCount : pendingUsers,
          },
        ]}
      />

      <div className="rounded-lg border bg-card">
        <div className="flex flex-col gap-3 border-b p-3 sm:flex-row sm:items-center sm:justify-between">
          <form onSubmit={applySearch} className="w-full min-w-0 sm:w-96">
            <div className="relative min-w-0">
              <MagnifyingGlassIcon
                className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                aria-hidden
              />
              <Input
                value={searchInput}
                onChange={(event) => setSearchInput(event.target.value)}
                placeholder={`Search ${title.toLowerCase()}`}
                className="rounded-lg pl-9"
              />
            </div>
          </form>
          {activeFilters && (
            <Button type="button" variant="ghost" className="rounded-lg" onClick={resetFilters}>
              <XIcon /> Clear
            </Button>
          )}
        </div>

        <DataTable
          columns={columns}
          data={paginatedUsers}
          getRowId={(user) => user.pid}
          isLoading={usersQuery.isLoading}
          isError={usersQuery.isError}
          errorTitle="People could not be loaded"
          onRetry={() => usersQuery.refetch()}
          emptyTitle={
            activeFilters ? `No matching ${title.toLowerCase()}` : `No ${title.toLowerCase()} found`
          }
          emptyDescription={
            activeFilters
              ? "Adjust or clear the current filters."
              : "People will appear here after users are added."
          }
        />

        {!usersQuery.isError && !usersQuery.isLoading && filteredUsers.length > 0 && (
          <div className="flex flex-col gap-3 border-t p-3 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>
              Page {page} of {totalPages} · {filteredUsers.length} {title.toLowerCase()}
            </p>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={page === 1 || usersQuery.isFetching}
                onClick={() => setPage((current) => current - 1)}
              >
                <CaretLeftIcon /> Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={page >= totalPages || usersQuery.isFetching}
                onClick={() => setPage((current) => current + 1)}
              >
                Next <CaretRightIcon />
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
