"use client";

import {
  ArrowClockwiseIcon,
  CaretDownIcon,
  KeyIcon,
  MagnifyingGlassIcon,
  ShieldCheckIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";

import { listPermissions, permissionsQueryKey, type Permission } from "#/api/permissions.ts";
import { listRoles, rolesQueryKey } from "#/api/roles.ts";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Input } from "#/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";

const ALL_ROLES = "__all__";

function splitPermissionName(name: string) {
  const [resource = "system", action = "access"] = name.split(":");
  return { resource, action };
}

function titleCase(value: string) {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatPermissionName(name: string) {
  const { resource, action } = splitPermissionName(name);
  return `${titleCase(action)} ${titleCase(resource)}`;
}

function actionBadgeClass(action: string) {
  switch (action.toLowerCase()) {
    case "read":
      return "bg-blue-50 text-blue-700 ring-blue-200";
    case "create":
      return "bg-green-50 text-green-700 ring-green-200";
    case "update":
      return "bg-amber-50 text-amber-700 ring-amber-200";
    case "delete":
      return "bg-red-50 text-red-700 ring-red-200";
    default:
      return "bg-muted text-muted-foreground ring-border";
  }
}

function groupPermissions(permissions: Permission[]) {
  return permissions.reduce<Record<string, Permission[]>>((groups, permission) => {
    const { resource } = splitPermissionName(permission.name);
    groups[resource] = groups[resource] ?? [];
    groups[resource].push(permission);
    return groups;
  }, {});
}

export default function PermissionsPage() {
  const [query, setQuery] = useState("");
  const [selectedRole, setSelectedRole] = useState(ALL_ROLES);
  const roleFilter = selectedRole === ALL_ROLES ? undefined : selectedRole;

  const permissionsQuery = useQuery({
    queryKey: [...permissionsQueryKey, roleFilter ?? "all"],
    queryFn: () => listPermissions(roleFilter),
  });

  const rolesQuery = useQuery({
    queryKey: rolesQueryKey,
    queryFn: listRoles,
  });

  const permissions = permissionsQuery.data ?? [];
  const filteredPermissions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return permissions;

    return permissions.filter((permission) => {
      const label = formatPermissionName(permission.name).toLowerCase();
      const description = permission.description?.toLowerCase() ?? "";

      return (
        permission.name.toLowerCase().includes(needle) ||
        label.includes(needle) ||
        description.includes(needle)
      );
    });
  }, [permissions, query]);

  const groupedPermissions = useMemo(
    () => groupPermissions(filteredPermissions),
    [filteredPermissions],
  );
  const groupEntries = Object.entries(groupedPermissions).sort(([a], [b]) => a.localeCompare(b));

  return (
    <div className="w-full pb-10">
      <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Permissions</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Review permission grants available to roles across the admin system.
          </p>
        </div>

        <Button
          type="button"
          variant="outline"
          onClick={() => permissionsQuery.refetch()}
          disabled={permissionsQuery.isFetching}
        >
          <ArrowClockwiseIcon
            className={cn("size-4", permissionsQuery.isFetching && "animate-spin")}
          />
          Refresh
        </Button>
      </div>

      <div className="mb-4 grid gap-3 border-b pb-4 lg:grid-cols-[minmax(0,1fr)_16rem_auto] lg:items-center">
        <div className="relative">
          <MagnifyingGlassIcon
            className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search permissions"
            className="pl-9"
          />
        </div>

        <Select value={selectedRole} onValueChange={(value) => setSelectedRole(value ?? ALL_ROLES)}>
          <SelectTrigger className="w-full">
            <SelectValue placeholder="Filter by role" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL_ROLES}>All roles</SelectItem>
            {rolesQuery.data?.map((role) => (
              <SelectItem key={role.pid} value={role.name}>
                {titleCase(role.name)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div className="text-sm text-muted-foreground">
          {permissions.length} {permissions.length === 1 ? "permission" : "permissions"}
        </div>
      </div>

      <PermissionsContent
        groups={groupEntries}
        query={query}
        isLoading={permissionsQuery.isLoading}
        isError={permissionsQuery.isError}
        onRetry={() => permissionsQuery.refetch()}
      />
    </div>
  );
}

function PermissionsContent({
  groups,
  query,
  isLoading,
  isError,
  onRetry,
}: {
  groups: [string, Permission[]][];
  query: string;
  isLoading: boolean;
  isError: boolean;
  onRetry: () => void;
}) {
  if (isLoading) {
    return (
      <div className="grid gap-4 lg:grid-cols-2">
        {Array.from({ length: 4 }).map((_, index) => (
          <div key={index} className="rounded-lg border p-5">
            <div className="mb-5 flex items-center gap-3">
              <Skeleton className="size-10 rounded-lg" />
              <Skeleton className="h-5 w-36" />
            </div>
            <div className="grid gap-3">
              <Skeleton className="h-14 w-full rounded-lg" />
              <Skeleton className="h-14 w-full rounded-lg" />
              <Skeleton className="h-14 w-full rounded-lg" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (isError) {
    return (
      <ErrorState
        icon={<KeyIcon className="size-8" aria-hidden />}
        title="Permissions could not be loaded"
        description="Check your session and retry the request."
        onRetry={onRetry}
      />
    );
  }

  if (!groups.length) {
    return (
      <EmptyState
        icon={<KeyIcon className="size-8" aria-hidden />}
        title={query ? "No matching permissions" : "No permissions found"}
        description={
          query
            ? "Try a different search term or role filter."
            : "Seed permissions before reviewing access rules."
        }
      />
    );
  }

  return (
    <div className="grid gap-4 lg:grid-cols-2">
      {groups.map(([resource, permissions]) => (
        <PermissionGroup key={resource} resource={resource} permissions={permissions} />
      ))}
    </div>
  );
}

function PermissionGroup({
  resource,
  permissions,
}: {
  resource: string;
  permissions: Permission[];
}) {
  const [isOpen, setIsOpen] = useState(true);

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <section className="rounded-lg border bg-background p-5">
        <CollapsibleTrigger
          render={
            <button
              type="button"
              className="group flex w-full items-center justify-between gap-4 text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/30"
            />
          }
        >
          <div className="flex min-w-0 items-center gap-3">
            <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-blue-600 text-white">
              <SquaresFourIcon className="size-5" aria-hidden />
            </span>
            <div className="min-w-0">
              <h2 className="truncate text-lg font-semibold">{titleCase(resource)}</h2>
              <p className="text-sm text-muted-foreground">
                {permissions.length} {permissions.length === 1 ? "permission" : "permissions"}
              </p>
            </div>
          </div>

          <CaretDownIcon
            className={cn(
              "size-5 shrink-0 text-muted-foreground transition-transform",
              isOpen && "rotate-180",
            )}
            aria-hidden
          />
        </CollapsibleTrigger>

        <CollapsibleContent>
          <div className="mt-5 grid gap-3">
            {permissions.map((permission) => (
              <PermissionRow key={permission.pid} permission={permission} />
            ))}
          </div>
        </CollapsibleContent>
      </section>
    </Collapsible>
  );
}

function PermissionRow({ permission }: { permission: Permission }) {
  const { action } = splitPermissionName(permission.name);

  return (
    <div className="flex items-start gap-3 rounded-lg border bg-muted/20 p-3">
      <span className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-lg bg-background text-muted-foreground">
        <ShieldCheckIcon className="size-4" aria-hidden />
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="text-sm font-medium">{formatPermissionName(permission.name)}</h3>
          <span
            className={cn(
              "rounded-full px-2 py-0.5 text-xs font-medium ring-1",
              actionBadgeClass(action),
            )}
          >
            {titleCase(action)}
          </span>
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {permission.description || "No description"}
        </p>
        <p className="mt-2 text-xs text-muted-foreground">{permission.name}</p>
      </div>
    </div>
  );
}
