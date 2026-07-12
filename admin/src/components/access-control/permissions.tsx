"use client";

import {
  ArrowClockwiseIcon,
  CaretDownIcon,
  CheckCircleIcon,
  KeyIcon,
  MagnifyingGlassIcon,
  ShieldCheckIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { listPermissions, permissionsQueryKey, type Permission } from "#/api/permissions.ts";
import { assignPermissionToRole, listRoles, rolesQueryKey, type Role } from "#/api/roles.ts";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { PageHeader } from "#/components/page-header";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
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

function groupPermissions(permissions: Permission[]) {
  return permissions.reduce<Record<string, Permission[]>>((groups, permission) => {
    const { resource } = splitPermissionName(permission.name);
    groups[resource] = groups[resource] ?? [];
    groups[resource].push(permission);
    return groups;
  }, {});
}

function roleLabel(role: Role) {
  return titleCase(role.name);
}

function percentage(value: number, total: number) {
  if (!total) return 0;
  return Math.round((value / total) * 100);
}

export default function PermissionsPage() {
  const queryClient = useQueryClient();
  const [query, setQuery] = useState("");
  const [selectedRole, setSelectedRole] = useState(ALL_ROLES);
  const [selectedPermissionIds, setSelectedPermissionIds] = useState<Set<number>>(new Set());
  const roleFilter = selectedRole === ALL_ROLES ? undefined : selectedRole;

  const allPermissionsQuery = useQuery({
    queryKey: [...permissionsQueryKey, "all"],
    queryFn: () => listPermissions(),
  });

  const rolesQuery = useQuery({
    queryKey: rolesQueryKey,
    queryFn: listRoles,
  });

  const selectedRoleModel = rolesQuery.data?.find((role) => role.name === roleFilter);
  const isAssignmentMode = !!selectedRoleModel;

  const assignedPermissionsQuery = useQuery({
    queryKey: [...permissionsQueryKey, "role", roleFilter ?? "none"],
    queryFn: () => listPermissions(roleFilter),
    enabled: isAssignmentMode,
  });

  const assignedPermissionIds = useMemo(
    () => new Set((assignedPermissionsQuery.data ?? []).map((permission) => permission.id)),
    [assignedPermissionsQuery.data],
  );

  const permissions = allPermissionsQuery.data ?? [];
  const resources = useMemo(() => groupPermissions(permissions), [permissions]);
  const pendingPermissionIds = useMemo(
    () =>
      [...selectedPermissionIds].filter((permissionId) => !assignedPermissionIds.has(permissionId)),
    [assignedPermissionIds, selectedPermissionIds],
  );

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
  const coverage = percentage(assignedPermissionIds.size, permissions.length);

  const assignmentMutation = useMutation({
    mutationFn: async () => {
      if (!selectedRoleModel) return [];

      return Promise.all(
        pendingPermissionIds.map((permissionId) =>
          assignPermissionToRole({
            roleId: selectedRoleModel.id,
            permissionId,
          }),
        ),
      );
    },
    onSuccess: async (assignments) => {
      await queryClient.invalidateQueries({ queryKey: permissionsQueryKey });
      setSelectedPermissionIds(new Set());

      toast.success(
        `${assignments.length} ${assignments.length === 1 ? "permission" : "permissions"} assigned to ${selectedRoleModel ? roleLabel(selectedRoleModel) : "role"}`,
        { id: "role-permissions-assigned" },
      );
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "role-permissions-assign-error",
      });
    },
  });

  useEffect(() => {
    setSelectedPermissionIds(new Set());
  }, [selectedRole]);

  function togglePermission(permissionId: number, checked: boolean) {
    setSelectedPermissionIds((current) => {
      const next = new Set(current);

      if (checked) next.add(permissionId);
      else next.delete(permissionId);

      return next;
    });
  }

  function refresh() {
    void allPermissionsQuery.refetch();
    void rolesQuery.refetch();
    void assignedPermissionsQuery.refetch();
  }

  async function submitAssignments() {
    await assignmentMutation.mutateAsync();
  }

  const isRefreshing = allPermissionsQuery.isFetching || assignedPermissionsQuery.isFetching;

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Permissions"
        subtitle="Define the actions each role can perform across the admin system."
        actions={
          <Button
            type="button"
            variant="outline"
            className="rounded-lg"
            onClick={refresh}
            disabled={isRefreshing}
          >
            <ArrowClockwiseIcon className={cn("size-4", isRefreshing && "animate-spin")} />
            Refresh
          </Button>
        }
      />

      <div className="grid rounded-lg border border-b-0 bg-card sm:grid-cols-3">
        <OverviewStat
          icon={<KeyIcon className="size-5" />}
          label="Permissions"
          value={permissions.length.toLocaleString()}
          detail="Available actions"
        />
        <OverviewStat
          icon={<SquaresFourIcon className="size-5" />}
          label="Resources"
          value={Object.keys(resources).length.toLocaleString()}
          detail="Protected areas"
        />
        <OverviewStat
          icon={<ShieldCheckIcon className="size-5" />}
          label={isAssignmentMode ? `${roleLabel(selectedRoleModel)} coverage` : "Role coverage"}
          value={isAssignmentMode ? `${coverage}%` : "Not selected"}
          detail={
            isAssignmentMode
              ? `${assignedPermissionIds.size} of ${permissions.length} assigned`
              : "Select a role to review"
          }
          last
        />
      </div>

      <section className="rounded-lg border bg-card">
        <div className="flex flex-col gap-1 border-b px-5 py-4 md:flex-row md:items-center md:justify-between">
          <div>
            <h2 className="font-semibold">Permission catalogue</h2>
            <p className="mt-0.5 text-sm text-muted-foreground">
              Browse every permission or select a role to assign additional access.
            </p>
          </div>
          <p className="text-sm font-medium text-muted-foreground">
            {filteredPermissions.length} shown
          </p>
        </div>

        <div className="grid gap-4 border-b bg-muted/20 p-4 lg:grid-cols-[minmax(18rem,1fr)_16rem_auto] lg:items-end">
          <div className="grid gap-1.5">
            <label htmlFor="permission-search" className="text-xs font-medium text-foreground">
              Search permissions
            </label>
            <div className="relative">
              <MagnifyingGlassIcon
                className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                aria-hidden
              />
              <Input
                id="permission-search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search by action, resource, or description"
                className="rounded-lg bg-background pl-9"
              />
            </div>
          </div>

          <div className="grid gap-1.5">
            <label className="text-xs font-medium text-foreground">Review access for</label>
            <Select
              value={selectedRole}
              onValueChange={(value) => setSelectedRole(value ?? ALL_ROLES)}
            >
              <SelectTrigger className="w-full rounded-lg bg-background">
                <SelectValue placeholder="Select a role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={ALL_ROLES}>All permissions</SelectItem>
                {rolesQuery.data?.map((role) => (
                  <SelectItem key={role.pid} value={role.name}>
                    {titleCase(role.name)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <Button
            type="button"
            className="rounded-lg"
            disabled={
              !isAssignmentMode || !pendingPermissionIds.length || assignmentMutation.isPending
            }
            onClick={submitAssignments}
          >
            <CheckCircleIcon className="size-4" />
            {assignmentMutation.isPending
              ? "Assigning..."
              : pendingPermissionIds.length
                ? `Assign ${pendingPermissionIds.length}`
                : "Assign permissions"}
          </Button>
        </div>

        {isAssignmentMode && (
          <div className="flex flex-col gap-3 border-b bg-primary/5 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-center gap-3">
              <span className="flex size-8 items-center justify-center bg-primary text-primary-foreground">
                <ShieldCheckIcon className="size-4" weight="fill" />
              </span>
              <div>
                <p className="text-sm font-medium">
                  Assigning access to {roleLabel(selectedRoleModel)}
                </p>
                <p className="text-xs text-muted-foreground">
                  Assigned permissions are locked; select any additional permissions to grant.
                </p>
              </div>
            </div>
            <Badge variant="outline" className="border-primary/25 bg-background text-primary">
              {assignedPermissionIds.size} assigned, {pendingPermissionIds.length} selected
            </Badge>
          </div>
        )}

        <div className="p-4 sm:p-5">
          <PermissionsContent
            groups={groupEntries}
            query={query}
            isLoading={
              allPermissionsQuery.isLoading ||
              rolesQuery.isLoading ||
              (isAssignmentMode && assignedPermissionsQuery.isLoading)
            }
            isError={
              allPermissionsQuery.isError ||
              rolesQuery.isError ||
              (isAssignmentMode && assignedPermissionsQuery.isError)
            }
            isAssignmentMode={isAssignmentMode}
            assignedPermissionIds={assignedPermissionIds}
            selectedPermissionIds={selectedPermissionIds}
            onTogglePermission={togglePermission}
            onRetry={refresh}
          />
        </div>
      </section>
    </div>
  );
}

function OverviewStat({
  icon,
  label,
  value,
  detail,
  last = false,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  detail: string;
  last?: boolean;
}) {
  return (
    <div
      className={cn("flex items-center gap-4 border-b p-5 sm:border-r", last && "sm:border-r-0")}
    >
      <span className="flex size-10 shrink-0 items-center justify-center bg-primary/10 text-primary">
        {icon}
      </span>
      <div className="min-w-0">
        <p className="text-xs font-medium text-muted-foreground">{label}</p>
        <div className="mt-1 flex items-baseline gap-2">
          <span className="text-2xl font-semibold tracking-tight">{value}</span>
          <span className="truncate text-xs text-muted-foreground">{detail}</span>
        </div>
      </div>
    </div>
  );
}

function PermissionsContent({
  groups,
  query,
  isLoading,
  isError,
  isAssignmentMode,
  assignedPermissionIds,
  selectedPermissionIds,
  onTogglePermission,
  onRetry,
}: {
  groups: [string, Permission[]][];
  query: string;
  isLoading: boolean;
  isError: boolean;
  isAssignmentMode: boolean;
  assignedPermissionIds: Set<number>;
  selectedPermissionIds: Set<number>;
  onTogglePermission: (permissionId: number, checked: boolean) => void;
  onRetry: () => void;
}) {
  if (isLoading) {
    return (
      <div className="grid gap-4 lg:grid-cols-2">
        {Array.from({ length: 4 }).map((_, index) => (
          <div key={index} className="rounded-lg border p-5">
            <div className="mb-5 flex items-center gap-3">
              <Skeleton className="size-10 rounded-lg" />
              <div className="grid gap-2">
                <Skeleton className="h-4 w-36 rounded-lg" />
                <Skeleton className="h-3 w-24 rounded-lg" />
              </div>
            </div>
            <div className="grid gap-3">
              <Skeleton className="h-16 w-full rounded-lg" />
              <Skeleton className="h-16 w-full rounded-lg" />
              <Skeleton className="h-16 w-full rounded-lg" />
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
    <div className="grid items-start gap-4 lg:grid-cols-2">
      {groups.map(([resource, permissions]) => (
        <PermissionGroup
          key={resource}
          resource={resource}
          permissions={permissions}
          isAssignmentMode={isAssignmentMode}
          assignedPermissionIds={assignedPermissionIds}
          selectedPermissionIds={selectedPermissionIds}
          onTogglePermission={onTogglePermission}
        />
      ))}
    </div>
  );
}

function PermissionGroup({
  resource,
  permissions,
  isAssignmentMode,
  assignedPermissionIds,
  selectedPermissionIds,
  onTogglePermission,
}: {
  resource: string;
  permissions: Permission[];
  isAssignmentMode: boolean;
  assignedPermissionIds: Set<number>;
  selectedPermissionIds: Set<number>;
  onTogglePermission: (permissionId: number, checked: boolean) => void;
}) {
  const [isOpen, setIsOpen] = useState(true);
  const assignedCount = isAssignmentMode
    ? permissions.filter((permission) => assignedPermissionIds.has(permission.id)).length
    : permissions.length;
  const groupCoverage = percentage(assignedCount, permissions.length);

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <section className="rounded-lg border bg-background">
        <CollapsibleTrigger
          render={
            <button
              type="button"
              className="group flex w-full items-center justify-between gap-4 border-b px-4 py-3.5 text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/30"
            />
          }
        >
          <div className="flex min-w-0 items-center gap-3">
            <span className="flex size-9 shrink-0 items-center justify-center bg-slate-900 text-white dark:bg-slate-100 dark:text-slate-900">
              <SquaresFourIcon className="size-4" weight="fill" aria-hidden />
            </span>
            <div className="min-w-0">
              <h2 className="truncate text-sm font-semibold">{titleCase(resource)}</h2>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {isAssignmentMode
                  ? `${assignedCount} of ${permissions.length} assigned`
                  : `${permissions.length} ${permissions.length === 1 ? "permission" : "permissions"}`}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {isAssignmentMode && (
              <span className="hidden text-xs font-medium text-muted-foreground sm:inline">
                {groupCoverage}% coverage
              </span>
            )}
            <CaretDownIcon
              className={cn(
                "size-4 shrink-0 text-muted-foreground transition-transform",
                isOpen && "rotate-180",
              )}
              aria-hidden
            />
          </div>
        </CollapsibleTrigger>

        <CollapsibleContent>
          <div className="divide-y px-4">
            {permissions.map((permission) => (
              <PermissionRow
                key={permission.pid}
                permission={permission}
                isAssignmentMode={isAssignmentMode}
                isAssigned={assignedPermissionIds.has(permission.id)}
                isSelected={selectedPermissionIds.has(permission.id)}
                onToggle={(checked) => onTogglePermission(permission.id, checked)}
              />
            ))}
          </div>
        </CollapsibleContent>
      </section>
    </Collapsible>
  );
}

function PermissionRow({
  permission,
  isAssignmentMode,
  isAssigned,
  isSelected,
  onToggle,
}: {
  permission: Permission;
  isAssignmentMode: boolean;
  isAssigned: boolean;
  isSelected: boolean;
  onToggle: (checked: boolean) => void;
}) {
  const { action } = splitPermissionName(permission.name);
  const selected = isAssigned || isSelected;

  const content = (
    <>
      {isAssignmentMode ? (
        <Checkbox
          className="rounded-lg"
          checked={selected}
          disabled={isAssigned}
          onCheckedChange={(checked) => onToggle(checked === true)}
          aria-label={formatPermissionName(permission.name)}
        />
      ) : (
        <span className="flex size-7 shrink-0 items-center justify-center bg-muted text-muted-foreground">
          <ShieldCheckIcon className="size-3.5" aria-hidden />
        </span>
      )}

      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <h3 className="text-sm font-medium">{formatPermissionName(permission.name)}</h3>
          {isAssigned && (
            <span className="inline-flex items-center gap-1 text-[11px] font-medium text-primary">
              <CheckCircleIcon className="size-3" weight="fill" /> Assigned
            </span>
          )}
        </div>
        <p className="mt-1 line-clamp-2 text-xs leading-5 text-muted-foreground">
          {permission.description || "No description provided."}
        </p>
        <code className="mt-1.5 block text-[10px] text-muted-foreground/70">{permission.name}</code>
      </div>

      <Badge
        variant="outline"
        className="self-start border-border bg-muted/40 text-muted-foreground"
      >
        {titleCase(action)}
      </Badge>
    </>
  );

  if (isAssignmentMode && !isAssigned) {
    return (
      <label
        className={cn(
          "flex cursor-pointer items-start gap-3 py-3.5 transition-colors hover:bg-muted/30",
          isSelected && "bg-primary/5",
        )}
      >
        {content}
      </label>
    );
  }

  return (
    <div className={cn("flex items-start gap-3 py-3.5", isAssigned && "bg-primary/5")}>
      {content}
    </div>
  );
}
