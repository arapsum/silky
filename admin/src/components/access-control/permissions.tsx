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
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { listPermissions, permissionsQueryKey, type Permission } from "#/api/permissions.ts";
import { assignPermissionToRole, listRoles, rolesQueryKey, type Role } from "#/api/roles.ts";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/page-header";
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

function actionBadgeClass(action: string) {
  switch (action.toLowerCase()) {
    case "read":
      return "bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950 dark:text-blue-300 dark:ring-blue-900";
    case "create":
      return "bg-green-50 text-green-700 ring-green-200 dark:bg-green-950 dark:text-green-300 dark:ring-green-900";
    case "update":
      return "bg-amber-50 text-amber-700 ring-amber-200 dark:bg-amber-950 dark:text-amber-300 dark:ring-amber-900";
    case "delete":
      return "bg-red-50 text-red-700 ring-red-200 dark:bg-red-950 dark:text-red-300 dark:ring-red-900";
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

function roleLabel(role: Role) {
  return titleCase(role.name);
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
  const permissionsCountLabel = isAssignmentMode
    ? `${assignedPermissionIds.size} of ${permissions.length} assigned`
    : `${permissions.length} ${permissions.length === 1 ? "permission" : "permissions"}`;

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

      if (checked) {
        next.add(permissionId);
      } else {
        next.delete(permissionId);
      }

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

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Permissions"
        subtitle="Review permission grants available to roles across the admin system."
        actions={
          <Button
            type="button"
            variant="outline"
            onClick={refresh}
            disabled={allPermissionsQuery.isFetching || assignedPermissionsQuery.isFetching}
          >
            <ArrowClockwiseIcon
              className={cn(
                "size-4",
                (allPermissionsQuery.isFetching || assignedPermissionsQuery.isFetching) &&
                  "animate-spin",
              )}
            />
            Refresh
          </Button>
        }
      />

      <div className="mb-4 grid gap-3 border-b pb-4 xl:grid-cols-[minmax(0,1fr)_16rem_auto_auto] xl:items-center">
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

        <div className="text-sm text-muted-foreground">{permissionsCountLabel}</div>

        <Button
          type="button"
          className="bg-blue-600 hover:bg-blue-700"
          disabled={
            !isAssignmentMode || !pendingPermissionIds.length || assignmentMutation.isPending
          }
          onClick={submitAssignments}
        >
          <CheckCircleIcon className="size-4" />
          {assignmentMutation.isPending ? "Assigning..." : `Assign ${pendingPermissionIds.length}`}
        </Button>
      </div>

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
    <div className="columns-1 gap-4 lg:columns-2">
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

  return (
    <Collapsible className="mb-4 break-inside-avoid" open={isOpen} onOpenChange={setIsOpen}>
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
                {isAssignmentMode
                  ? `${assignedCount} of ${permissions.length} assigned`
                  : `${permissions.length} ${permissions.length === 1 ? "permission" : "permissions"}`}
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
  const content = (
    <>
      {isAssignmentMode && (
        <Checkbox
          className="mt-1"
          checked={isAssigned || isSelected}
          disabled={isAssigned}
          onCheckedChange={(checked) => onToggle(checked === true)}
          aria-label={formatPermissionName(permission.name)}
        />
      )}

      {!isAssignmentMode && (
        <span className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-lg bg-background text-muted-foreground">
          <ShieldCheckIcon className="size-4" aria-hidden />
        </span>
      )}

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
          {isAssigned && (
            <span className="rounded-full bg-green-50 px-2 py-0.5 text-xs font-medium text-green-700 ring-1 ring-green-200 dark:bg-green-950 dark:text-green-300 dark:ring-green-900">
              Assigned
            </span>
          )}
        </div>
        <p className="mt-1 text-sm text-muted-foreground">
          {permission.description || "No description"}
        </p>
        <p className="mt-2 text-xs text-muted-foreground">{permission.name}</p>
      </div>
    </>
  );

  if (isAssignmentMode) {
    return (
      <label
        className={cn(
          "flex items-start gap-3 rounded-lg border bg-muted/20 p-3",
          isAssigned && "bg-green-50/60 dark:bg-green-950/20",
        )}
      >
        {content}
      </label>
    );
  }

  return <div className="flex items-start gap-3 rounded-lg border bg-muted/20 p-3">{content}</div>;
}
