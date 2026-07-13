"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  DotsThreeIcon,
  MagnifyingGlassIcon,
  PencilSimpleIcon,
  PlusIcon,
  ShieldCheckIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import {
  createRole,
  listRoles,
  rolesQueryKey,
  updateRole,
  type Role,
  type RoleInput,
  type RoleUser,
} from "#/api/roles.ts";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import FormField from "#/components/form-field";
import { initials, titleCase } from "#/utils/formatters";
import { RefreshButton } from "#/components/refresh-button";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/page-header";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";
import { useEffect, useMemo, useState } from "react";

const roleSchema = z.object({
  name: z
    .string()
    .trim()
    .min(2, "Name requires 2 letters")
    .max(32, "Name must be under 32 letters")
    .regex(/^[a-zA-Z0-9_ ]+$/, "Only letters, numbers and underscores can be used."),
  description: z.string().trim().max(256, "Description must be under 256 characters").optional(),
});

type RoleValues = z.infer<typeof roleSchema>;

type RoleDialogMode = { type: "create"; role?: never } | { type: "edit"; role: Role };

function rolePayload(values: RoleValues, preserveEmptyDescription = false): RoleInput {
  const description = values.description?.trim() ?? "";

  return {
    name: values.name.trim(),
    ...(description || preserveEmptyDescription ? { description } : {}),
  };
}

export default function RolesPage() {
  const [query, setQuery] = useState("");
  const [dialogMode, setDialogMode] = useState<RoleDialogMode>();

  const rolesQuery = useQuery({
    queryKey: rolesQueryKey,
    queryFn: listRoles,
  });

  const roles = rolesQuery.data ?? [];
  const filteredRoles = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return roles;

    return roles.filter((role) => {
      const description = role.description?.toLowerCase() ?? "";
      return role.name.toLowerCase().includes(needle) || description.includes(needle);
    });
  }, [query, roles]);

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Roles"
        subtitle="Manage access roles used to group permissions across the admin system."
        actions={
          <>
            <RefreshButton
              onRefresh={() => void rolesQuery.refetch()}
              isRefreshing={rolesQuery.isFetching}
            />
            <Button type="button" onClick={() => setDialogMode({ type: "create" })}>
              <PlusIcon className="size-4" />
              New Role
            </Button>
          </>
        }
      />

      <div className="mb-4 flex flex-col gap-3 border-b pb-4 md:flex-row md:items-center md:justify-between">
        <div className="relative w-full md:max-w-sm">
          <MagnifyingGlassIcon
            className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search roles"
            className="pl-9"
          />
        </div>

        <div className="text-sm text-muted-foreground">
          {roles.length} {roles.length === 1 ? "role" : "roles"}
        </div>
      </div>

      <RolesGrid
        roles={filteredRoles}
        isLoading={rolesQuery.isLoading}
        isError={rolesQuery.isError}
        query={query}
        onEdit={(role) => setDialogMode({ type: "edit", role })}
        onRetry={() => rolesQuery.refetch()}
      />

      <RoleDialog mode={dialogMode} onOpenChange={(open) => !open && setDialogMode(undefined)} />
    </div>
  );
}

function RolesGrid({
  roles,
  isLoading,
  isError,
  query,
  onEdit,
  onRetry,
}: {
  roles: Role[];
  isLoading: boolean;
  isError: boolean;
  query: string;
  onEdit: (role: Role) => void;
  onRetry: () => void;
}) {
  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 8 }).map((_, index) => (
          <div key={index} className="min-h-56 rounded-lg border p-5">
            <div className="mb-6 flex items-center gap-3">
              <Skeleton className="size-10 rounded-lg" />
              <Skeleton className="h-5 w-32" />
            </div>
            <Skeleton className="mb-8 h-6 w-40" />
            <Skeleton className="h-10 w-full rounded-lg" />
          </div>
        ))}
      </div>
    );
  }

  if (isError) {
    return (
      <ErrorState
        icon={<ShieldCheckIcon className="size-8" aria-hidden />}
        title="Roles could not be loaded"
        description="Check your session and retry the request."
        onRetry={onRetry}
      />
    );
  }

  if (!roles.length) {
    return (
      <EmptyState
        icon={<ShieldCheckIcon className="size-8" aria-hidden />}
        title={query ? "No matching roles" : "No roles yet"}
        description={
          query
            ? "Try a different search term."
            : "Create the first role to start building your access control model."
        }
      />
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      {roles.map((role, index) => (
        <RoleCard key={role.pid} role={role} toneIndex={index} onEdit={() => onEdit(role)} />
      ))}
    </div>
  );
}

const roleTones = [
  "bg-blue-600 text-white",
  "bg-amber-500 text-white",
  "bg-red-500 text-white",
  "bg-violet-500 text-white",
  "bg-green-600 text-white",
  "bg-indigo-600 text-white",
];

function RoleCard({
  role,
  toneIndex,
  onEdit,
}: {
  role: Role;
  toneIndex: number;
  onEdit: () => void;
}) {
  const shownUsers = role.users.slice(0, 4);
  const remainingUsers = Math.max(0, role.users.length - shownUsers.length);

  return (
    <div className="grid min-h-56 grid-rows-[auto_minmax(2.5rem,auto)_2.25rem_auto] gap-4 rounded-lg border bg-background p-5">
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <span
            className={cn(
              "flex size-10 shrink-0 items-center justify-center rounded-lg",
              roleTones[toneIndex % roleTones.length],
            )}
          >
            <ShieldCheckIcon className="size-5" aria-hidden />
          </span>
          <h2 className="truncate text-lg font-semibold">{titleCase(role.name)}</h2>
        </div>

        <Button type="button" variant="ghost" size="icon-sm" aria-label="Role actions">
          <DotsThreeIcon className="size-5" aria-hidden />
        </Button>
      </div>

      <p className="line-clamp-2 text-sm text-muted-foreground">
        {role.description || "No description"}
      </p>

      <div className="flex min-h-8 items-center gap-3">
        <AvatarStack users={shownUsers} remaining={remainingUsers} />
        <span className="truncate text-sm text-muted-foreground">
          Total {role.users.length} {role.users.length === 1 ? "user" : "users"}
        </span>
      </div>

      <Button type="button" variant="outline" className="w-full" onClick={onEdit}>
        <PencilSimpleIcon className="size-4" />
        Edit Role
      </Button>
    </div>
  );
}

function AvatarStack({ users, remaining }: { users: RoleUser[]; remaining: number }) {
  if (!users.length) {
    return (
      <span className="flex h-7 items-center text-xs text-muted-foreground">No users assigned</span>
    );
  }

  return (
    <div className="flex -space-x-2">
      {users.map((user) => (
        <Avatar key={user.pid} size="sm" className="ring-2 ring-background">
          <AvatarImage src={user.image ?? undefined} alt={user.name} />
          <AvatarFallback>{initials(user.name)}</AvatarFallback>
        </Avatar>
      ))}
      {remaining > 0 && (
        <span className="flex size-6 items-center justify-center rounded-full bg-muted text-xs text-muted-foreground ring-2 ring-background">
          +{remaining}
        </span>
      )}
    </div>
  );
}

function RoleDialog({
  mode,
  onOpenChange,
}: {
  mode?: RoleDialogMode;
  onOpenChange: (open: boolean) => void;
}) {
  const queryClient = useQueryClient();
  const form = useForm<RoleValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: "",
      description: "",
    },
  });
  const isOpen = !!mode;
  const isEditing = mode?.type === "edit";

  useEffect(() => {
    if (!mode) return;

    form.reset({
      name: mode.type === "edit" ? mode.role.name : "",
      description: mode.type === "edit" ? (mode.role.description ?? "") : "",
    });
  }, [form, mode]);

  const mutation = useMutation({
    mutationFn: async (values: RoleValues) => {
      if (mode?.type === "edit") {
        return updateRole(mode.role.pid, rolePayload(values, true));
      }

      return createRole(rolePayload(values));
    },
    onSuccess: async (role) => {
      await queryClient.invalidateQueries({ queryKey: rolesQueryKey });
      toast.success(isEditing ? `Updated ${role.name}` : `Created ${role.name}`, {
        id: "role-save-success",
      });
      onOpenChange(false);
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "role-save-error",
      });
    },
  });

  async function onSubmit(values: RoleValues) {
    await mutation.mutateAsync(values);
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit role" : "Create role"}</DialogTitle>
          <DialogDescription>
            Roles group permissions and describe what a user can do in Silk.
          </DialogDescription>
        </DialogHeader>

        <form className="grid gap-5" onSubmit={form.handleSubmit(onSubmit)} noValidate>
          <FormField
            control={form.control}
            name="name"
            label="Name"
            placeholder="Administrator"
            autoComplete="off"
            required
          />

          <FormField
            control={form.control}
            name="description"
            label="Description"
            type="textarea"
            placeholder="Describe what this role is responsible for"
          />

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              disabled={mutation.isPending}
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "Saving..." : "Save Role"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
