"use client";

import {
  ArrowClockwiseIcon,
  EyeIcon,
  ImageSquareIcon,
  KeyIcon,
  MagnifyingGlassIcon,
  PackageIcon,
  PencilSimpleIcon,
  PlusIcon,
  ShieldCheckIcon,
  ShoppingBagIcon,
  TagIcon,
  TrashIcon,
  UsersThreeIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { listPermissions, permissionsQueryKey, type Permission } from "#/api/permissions.ts";
import {
  assignPermissionToRole,
  listRoles,
  revokePermissionFromRole,
  rolesQueryKey,
  type Role,
} from "#/api/roles.ts";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { PageHeader } from "#/components/page-header";
import { Button } from "#/components/ui/button";
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

function resourceIcon(resource: string) {
  switch (resource) {
    case "products":
      return <PackageIcon />;
    case "categories":
      return <TagIcon />;
    case "orders":
      return <ShoppingBagIcon />;
    case "users":
      return <UsersThreeIcon />;
    case "media":
      return <ImageSquareIcon />;
    case "roles":
      return <ShieldCheckIcon />;
    default:
      return <KeyIcon />;
  }
}

function actionIcon(action: string) {
  switch (action) {
    case "read":
      return <EyeIcon />;
    case "create":
      return <PlusIcon />;
    case "update":
      return <PencilSimpleIcon />;
    case "delete":
      return <TrashIcon />;
    default:
      return <KeyIcon />;
  }
}

function permissionLabel(permission: Permission) {
  const { action, resource } = splitPermissionName(permission.name);
  return `${titleCase(action)} ${titleCase(resource)}`;
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
  const roles = rolesQuery.data ?? [];
  const rolePermissionQueries = useQueries({
    queries: roles.map((role) => ({
      queryKey: [...permissionsQueryKey, "role", role.name],
      queryFn: () => listPermissions(role.name),
      enabled: rolesQuery.isSuccess,
    })),
  });

  const permissions = allPermissionsQuery.data ?? [];
  const selectedRoleModel = roles.find((role) => role.name === roleFilter);
  const isAssignmentMode = Boolean(selectedRoleModel);
  const assignedByRole = useMemo(
    () =>
      new Map(
        roles.map((role, index) => [
          role.name,
          new Set((rolePermissionQueries[index]?.data ?? []).map((permission) => permission.id)),
        ]),
      ),
    [rolePermissionQueries, roles],
  );
  const assignedPermissionIds = assignedByRole.get(roleFilter ?? "") ?? new Set<number>();
  const permissionsToAssign = [...selectedPermissionIds].filter(
    (permissionId) => !assignedPermissionIds.has(permissionId),
  );
  const permissionsToRevoke = [...selectedPermissionIds].filter((permissionId) =>
    assignedPermissionIds.has(permissionId),
  );
  const pendingPermissionCount = permissionsToAssign.length + permissionsToRevoke.length;
  const filteredPermissions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return permissions;

    return permissions.filter((permission) => {
      const { resource, action } = splitPermissionName(permission.name);
      const description = permission.description?.toLowerCase() ?? "";
      return `${permission.name} ${resource} ${action} ${description}`
        .toLowerCase()
        .includes(needle);
    });
  }, [permissions, query]);
  const groups = useMemo<[string, Permission[]][]>(
    () =>
      Object.entries(groupPermissions(filteredPermissions))
        .map(([resource, entries]): [string, Permission[]] => [
          resource,
          [...entries].sort((a, b) => a.name.localeCompare(b.name)),
        ])
        .sort(([first], [second]) => first.localeCompare(second)),
    [filteredPermissions],
  );

  useEffect(() => {
    setSelectedPermissionIds(new Set());
  }, [selectedRole]);

  const permissionsMutation = useMutation({
    mutationFn: async () => {
      if (!selectedRoleModel) return [];

      const assignments = await Promise.all(
        permissionsToAssign.map((permissionId) =>
          assignPermissionToRole({ roleId: selectedRoleModel.id, permissionId }),
        ),
      );

      await Promise.all(
        permissionsToRevoke.map((permissionId) =>
          revokePermissionFromRole({ roleId: selectedRoleModel.id, permissionId }),
        ),
      );

      return assignments;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: permissionsQueryKey });
      setSelectedPermissionIds(new Set());
      const changes = [
        permissionsToAssign.length && `${permissionsToAssign.length} assigned`,
        permissionsToRevoke.length && `${permissionsToRevoke.length} revoked`,
      ]
        .filter(Boolean)
        .join(", ");
      toast.success(`${changes} for ${selectedRoleModel ? roleLabel(selectedRoleModel) : "role"}`, {
        id: "role-permissions-updated",
      });
    },
    onError: (error) => toast.error(error.message, { id: "role-permissions-update-error" }),
  });

  function togglePermission(permissionId: number, checked: boolean) {
    setSelectedPermissionIds((current) => {
      const next = new Set(current);
      if (checked) next.add(permissionId);
      else next.delete(permissionId);
      return next;
    });
  }

  function clearFilters() {
    setQuery("");
    setSelectedRole(ALL_ROLES);
  }

  function refresh() {
    void allPermissionsQuery.refetch();
    void rolesQuery.refetch();
    rolePermissionQueries.forEach((roleQuery) => void roleQuery.refetch());
  }

  const isLoading =
    allPermissionsQuery.isLoading ||
    rolesQuery.isLoading ||
    rolePermissionQueries.some((roleQuery) => roleQuery.isLoading);
  const isError =
    allPermissionsQuery.isError ||
    rolesQuery.isError ||
    rolePermissionQueries.some((roleQuery) => roleQuery.isError);
  const isRefreshing =
    allPermissionsQuery.isFetching ||
    rolesQuery.isFetching ||
    rolePermissionQueries.some((roleQuery) => roleQuery.isFetching);
  const hasFilters = Boolean(query || isAssignmentMode);

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Permissions"
        subtitle="Review each role's access across the admin system."
        actions={
          <Button
            type="button"
            variant="outline"
            className="rounded-lg"
            onClick={refresh}
            disabled={isRefreshing}
          >
            <ArrowClockwiseIcon className={cn(isRefreshing && "animate-spin")} />
            Refresh
          </Button>
        }
      />

      <section className="rounded-lg border bg-card">
        <div className="flex flex-col gap-3 border-b p-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="relative w-full min-w-0 sm:w-96">
            <MagnifyingGlassIcon
              className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
              aria-hidden
            />
            <Input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search permissions"
              className="rounded-lg pl-9"
            />
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <Select
              value={selectedRole}
              onValueChange={(value) => setSelectedRole(value ?? ALL_ROLES)}
            >
              <SelectTrigger className="w-full rounded-lg bg-background sm:w-52">
                <SelectValue placeholder="Select a role" />
              </SelectTrigger>
              <SelectContent className="rounded-lg">
                <SelectItem value={ALL_ROLES}>All roles</SelectItem>
                {roles.map((role) => (
                  <SelectItem key={role.pid} value={role.name}>
                    {roleLabel(role)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {hasFilters && (
              <Button type="button" variant="ghost" className="rounded-lg" onClick={clearFilters}>
                <XIcon /> Clear
              </Button>
            )}
            {isAssignmentMode && (
              <Button
                type="button"
                className="rounded-lg"
                disabled={!pendingPermissionCount || permissionsMutation.isPending}
                onClick={() => permissionsMutation.mutate()}
              >
                <PlusIcon />
                {permissionsMutation.isPending
                  ? "Saving..."
                  : pendingPermissionCount
                    ? `Save ${pendingPermissionCount} changes`
                    : "Manage permissions"}
              </Button>
            )}
          </div>
        </div>

        <PermissionsMatrix
          groups={groups}
          roles={roles}
          assignedByRole={assignedByRole}
          selectedRole={roleFilter}
          selectedPermissionIds={selectedPermissionIds}
          isLoading={isLoading}
          isError={isError}
          query={query}
          onTogglePermission={togglePermission}
          onRetry={refresh}
        />

        {!isLoading && !isError && groups.length > 0 && (
          <footer className="flex flex-col gap-3 border-t p-3 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>
              Showing {groups.length} {groups.length === 1 ? "resource" : "resources"} ·{" "}
              {filteredPermissions.length} permissions
            </p>
            {isAssignmentMode && selectedRoleModel && (
              <div className="flex flex-wrap gap-x-3 gap-y-1">
                <p>
                  {assignedPermissionIds.size} assigned to {roleLabel(selectedRoleModel)}
                </p>
                <p>Click a granted action to mark it for removal.</p>
              </div>
            )}
          </footer>
        )}
      </section>
    </div>
  );
}

function PermissionsMatrix({
  groups,
  roles,
  assignedByRole,
  selectedRole,
  selectedPermissionIds,
  isLoading,
  isError,
  query,
  onTogglePermission,
  onRetry,
}: {
  groups: [string, Permission[]][];
  roles: Role[];
  assignedByRole: Map<string, Set<number>>;
  selectedRole?: string;
  selectedPermissionIds: Set<number>;
  isLoading: boolean;
  isError: boolean;
  query: string;
  onTogglePermission: (permissionId: number, checked: boolean) => void;
  onRetry: () => void;
}) {
  if (isLoading) return <MatrixSkeleton />;

  if (isError) {
    return (
      <ErrorState
        icon={<KeyIcon className="size-8" aria-hidden />}
        title="Permissions could not be loaded"
        description="Check your session and retry the request."
        onRetry={onRetry}
        className="border-0"
      />
    );
  }

  if (!groups.length) {
    return (
      <EmptyState
        icon={<KeyIcon className="size-8" aria-hidden />}
        title={query ? "No matching permissions" : "No permissions found"}
        description={
          query ? "Try a different search term." : "Seed permissions before reviewing access rules."
        }
        className="border-0"
      />
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[58rem] border-collapse text-left">
        <thead className="bg-muted/25">
          <tr>
            <th className="w-56 border-b px-4 py-4 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Resource
            </th>
            {roles.map((role) => (
              <th key={role.pid} className="min-w-44 border-b border-l px-4 py-4 text-left">
                <p className="text-sm font-semibold">{roleLabel(role)}</p>
                <p className="mt-0.5 text-xs font-normal text-muted-foreground">
                  {role.users.length} {role.users.length === 1 ? "user" : "users"}
                </p>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {groups.map(([resource, permissions]) => (
            <tr key={resource} className="border-b last:border-b-0">
              <th scope="row" className="px-4 py-4 align-middle">
                <div className="flex items-center gap-3">
                  <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                    {resourceIcon(resource)}
                  </span>
                  <div>
                    <p className="text-sm font-semibold">{titleCase(resource)}</p>
                    <p className="mt-0.5 text-xs font-normal text-muted-foreground">
                      {permissions.length} {permissions.length === 1 ? "action" : "actions"}
                    </p>
                  </div>
                </div>
              </th>
              {roles.map((role) => (
                <td
                  key={role.pid}
                  className={cn(
                    "border-l px-4 py-4 align-middle",
                    selectedRole === role.name && "bg-primary/[0.035]",
                  )}
                >
                  <div className="flex flex-wrap gap-1.5">
                    {permissions.map((permission) => (
                      <PermissionAction
                        key={permission.pid}
                        permission={permission}
                        isGranted={assignedByRole.get(role.name)?.has(permission.id) ?? false}
                        isSelected={
                          selectedRole === role.name && selectedPermissionIds.has(permission.id)
                        }
                        isAssignable={selectedRole === role.name}
                        onToggle={onTogglePermission}
                      />
                    ))}
                  </div>
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function PermissionAction({
  permission,
  isGranted,
  isSelected,
  isAssignable,
  onToggle,
}: {
  permission: Permission;
  isGranted: boolean;
  isSelected: boolean;
  isAssignable: boolean;
  onToggle: (permissionId: number, checked: boolean) => void;
}) {
  const { action } = splitPermissionName(permission.name);
  const isActive = isGranted || isSelected;
  const isMarkedForRemoval = isGranted && isSelected;

  return (
    <Button
      type="button"
      variant="outline"
      size="icon-sm"
      className={cn(
        "rounded-lg disabled:opacity-100",
        isMarkedForRemoval
          ? "border-destructive/30 bg-destructive/10 text-destructive hover:bg-destructive/15"
          : isActive
            ? "border-primary/30 bg-primary/10 text-primary hover:bg-primary/15"
            : "border-border bg-background text-muted-foreground hover:bg-muted",
      )}
      title={permissionLabel(permission)}
      aria-label={permissionLabel(permission)}
      aria-pressed={isActive}
      disabled={!isAssignable}
      onClick={() => onToggle(permission.id, !isSelected)}
    >
      {actionIcon(action)}
    </Button>
  );
}

function MatrixSkeleton() {
  return (
    <div className="overflow-hidden">
      <div className="grid grid-cols-[14rem_repeat(4,minmax(10rem,1fr))] border-b">
        {Array.from({ length: 5 }).map((_, index) => (
          <Skeleton key={index} className="m-4 h-10 rounded-lg" />
        ))}
      </div>
      {Array.from({ length: 6 }).map((_, row) => (
        <div
          key={row}
          className="grid grid-cols-[14rem_repeat(4,minmax(10rem,1fr))] border-b last:border-b-0"
        >
          {Array.from({ length: 5 }).map((_, column) => (
            <Skeleton key={column} className="m-4 h-9 rounded-lg" />
          ))}
        </div>
      ))}
    </div>
  );
}
