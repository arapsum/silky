"use client";

import type { ColumnDef } from "@tanstack/react-table";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  ArrowClockwiseIcon,
  CaretLeftIcon,
  CaretRightIcon,
  MagnifyingGlassIcon,
  UserPlusIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState, type FormEvent } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { listRoles, rolesQueryKey, type Role } from "#/api/roles.ts";
import {
  assignRoleToUser,
  createStaffUser,
  listUsers,
  revokeRoleFromUser,
  usersQueryKey,
  type User,
} from "#/api/users.ts";
import { SummaryGrid } from "#/components/catalogue/summary-grid";
import { DataTable } from "#/components/data-table";
import FormField from "#/components/form-field";
import { PageHeader } from "#/components/page-header";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { cn } from "#/lib/utils";

type PeopleTablePageProps = {
  title: string;
  description: string;
  category: "customers" | "staff";
};

const PAGE_SIZE = 20;

const createStaffUserSchema = z
  .object({
    name: z
      .string()
      .trim()
      .min(6, "Name requires 6 letters")
      .max(32, "Name must be under 32 letters")
      .regex(/^[a-zA-Z0-9_ ]+$/, "Only letters, numbers and underscores can be used."),
    email: z.email("Invalid email address"),
    roleId: z.string().min(1, "Choose an initial role"),
    password: z
      .string()
      .min(8, "Password requires 8 characters")
      .max(48, "Password must be under 48 characters")
      .regex(/^\S+$/, "Password cannot have spaces")
      .refine((value) => !value.includes(","), "Password cannot have commas"),
    confirmPassword: z.string(),
  })
  .refine((values) => values.password === values.confirmPassword, {
    message: "Passwords do not match",
    path: ["confirmPassword"],
  });

type CreateStaffUserValues = z.infer<typeof createStaffUserSchema>;

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

function userColumns(onAssignRole: (user: User) => void): ColumnDef<User>[] {
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
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="rounded-lg"
          onClick={() => onAssignRole(row.original)}
        >
          <UserPlusIcon />
          Manage roles
        </Button>
      ),
      size: 160,
    },
  ];
}

export default function PeopleTablePage({ title, description, category }: PeopleTablePageProps) {
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [selectedUser, setSelectedUser] = useState<User>();
  const [isCreateStaffDialogOpen, setIsCreateStaffDialogOpen] = useState(false);
  const usersQuery = useQuery({
    queryKey: [...usersQueryKey, category],
    queryFn: () => listUsers(),
  });
  const rolesQuery = useQuery({
    queryKey: rolesQueryKey,
    queryFn: listRoles,
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

  const columns = useMemo(() => userColumns(setSelectedUser), []);
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
          <>
            <Button
              type="button"
              variant="outline"
              className="rounded-lg"
              onClick={() => usersQuery.refetch()}
              disabled={usersQuery.isFetching}
            >
              <ArrowClockwiseIcon
                className={cn("size-4", usersQuery.isFetching && "animate-spin")}
              />
              Refresh
            </Button>
            {category === "staff" && (
              <Button
                type="button"
                className="rounded-lg"
                onClick={() => setIsCreateStaffDialogOpen(true)}
              >
                <UserPlusIcon />
                Add staff member
              </Button>
            )}
          </>
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

      <AssignUserRoleDialog
        user={selectedUser}
        roles={rolesQuery.data ?? []}
        isLoading={rolesQuery.isLoading}
        onOpenChange={(open) => !open && setSelectedUser(undefined)}
        onRoleRevoked={(roleId) =>
          setSelectedUser((current) =>
            current
              ? { ...current, roles: current.roles.filter((role) => role.id !== roleId) }
              : undefined,
          )
        }
      />
      <CreateStaffUserDialog
        open={isCreateStaffDialogOpen}
        roles={rolesQuery.data ?? []}
        isLoadingRoles={rolesQuery.isLoading}
        onOpenChange={setIsCreateStaffDialogOpen}
      />
    </div>
  );
}

function CreateStaffUserDialog({
  open,
  roles,
  isLoadingRoles,
  onOpenChange,
}: {
  open: boolean;
  roles: Role[];
  isLoadingRoles: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const queryClient = useQueryClient();
  const staffRoles = roles.filter((role) => role.name.toLowerCase() !== "customer");
  const form = useForm<CreateStaffUserValues>({
    resolver: zodResolver(createStaffUserSchema),
    defaultValues: {
      name: "",
      email: "",
      roleId: "",
      password: "",
      confirmPassword: "",
    },
  });
  const staffRoleOptions = staffRoles.map((role) => ({
    label: titleCase(role.name),
    value: String(role.id),
  }));

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      form.reset();
    }

    onOpenChange(nextOpen);
  }

  const mutation = useMutation({
    mutationFn: (values: CreateStaffUserValues) => {
      return createStaffUser({
        name: values.name.trim(),
        email: values.email.trim(),
        password: values.password,
        confirmPassword: values.confirmPassword,
        roleId: Number(values.roleId),
      });
    },
    onSuccess: async (response) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: usersQueryKey }),
        queryClient.invalidateQueries({ queryKey: rolesQueryKey }),
      ]);
      toast.success(response.message, { id: "staff-user-created" });
      handleOpenChange(false);
    },
    onError: (error) => toast.error(error.message, { id: "staff-user-create-error" }),
  });

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="rounded-lg sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add staff member</DialogTitle>
          <DialogDescription>
            Create an internal account and assign its initial access role. Customer accounts are
            created through the storefront instead.
          </DialogDescription>
        </DialogHeader>

        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
          noValidate
        >
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="sm:col-span-2">
              <FormField
                control={form.control}
                name="name"
                label="Full name"
                autoComplete="name"
                className="rounded-lg"
                required
              />
            </div>
            <div className="sm:col-span-2">
              <FormField
                control={form.control}
                name="email"
                label="Work email"
                type="email"
                autoComplete="email"
                className="rounded-lg"
                required
              />
            </div>
            <div className="sm:col-span-2">
              <FormField
                control={form.control}
                name="roleId"
                label="Initial role"
                type="select"
                placeholder={isLoadingRoles ? "Loading roles..." : "Select a role"}
                options={staffRoleOptions}
                disabled={isLoadingRoles || !staffRoleOptions.length}
                className="rounded-lg bg-background"
                required
              />
              {!isLoadingRoles && !staffRoles.length && (
                <p className="text-sm text-muted-foreground">
                  Create a non-customer role before adding staff members.
                </p>
              )}
            </div>
            <div>
              <FormField
                control={form.control}
                name="password"
                label="Temporary password"
                type="password"
                autoComplete="new-password"
                className="rounded-lg"
                required
              />
            </div>
            <div>
              <FormField
                control={form.control}
                name="confirmPassword"
                label="Confirm password"
                type="password"
                autoComplete="new-password"
                className="rounded-lg"
                required
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              className="rounded-lg"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              className="rounded-lg"
              disabled={mutation.isPending || isLoadingRoles || !staffRoles.length}
            >
              <UserPlusIcon />
              {mutation.isPending ? "Creating..." : "Create staff account"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function AssignUserRoleDialog({
  user,
  roles,
  isLoading,
  onOpenChange,
  onRoleRevoked,
}: {
  user?: User;
  roles: Role[];
  isLoading: boolean;
  onOpenChange: (open: boolean) => void;
  onRoleRevoked: (roleId: number) => void;
}) {
  const queryClient = useQueryClient();
  const [roleId, setRoleId] = useState("");
  const assignableRoles = roles.filter(
    (role) => !user?.roles.some((assignedRole) => assignedRole.id === role.id),
  );
  const selectedRole = roles.find((role) => role.id === Number(roleId));

  useEffect(() => {
    setRoleId("");
  }, [user]);

  const mutation = useMutation({
    mutationFn: () => {
      if (!user || !roleId) {
        throw new Error("Choose a role to assign");
      }

      return assignRoleToUser({ userId: user.id, roleId: Number(roleId) });
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: usersQueryKey }),
        queryClient.invalidateQueries({ queryKey: rolesQueryKey }),
      ]);
      toast.success(
        `Assigned ${titleCase(roles.find((role) => role.id === Number(roleId))?.name ?? "role")} to ${user?.name}`,
        {
          id: "user-role-assigned",
        },
      );
      onOpenChange(false);
    },
    onError: (error) => toast.error(error.message, { id: "user-role-assignment-error" }),
  });

  const revokeMutation = useMutation({
    mutationFn: (role: User["roles"][number]) => {
      if (!user) {
        throw new Error("Select a user before revoking a role");
      }

      return revokeRoleFromUser({ userId: user.id, roleId: role.id });
    },
    onSuccess: async (_, role) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: usersQueryKey }),
        queryClient.invalidateQueries({ queryKey: rolesQueryKey }),
      ]);
      onRoleRevoked(role.id);
      toast.success(`Revoked ${titleCase(role.name)} from ${user?.name}`, {
        id: "user-role-revoked",
      });
    },
    onError: (error) => toast.error(error.message, { id: "user-role-revoke-error" }),
  });

  return (
    <Dialog open={Boolean(user)} onOpenChange={onOpenChange}>
      <DialogContent className="rounded-lg sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Manage roles</DialogTitle>
          <DialogDescription>
            Assign or revoke the access roles held by {user?.name ?? "this user"}.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="space-y-2">
            <p className="text-sm font-medium">Current roles</p>
            {user?.roles.length ? (
              <div className="flex flex-wrap gap-2">
                {user.roles.map((role) => (
                  <div
                    key={role.pid}
                    className="flex items-center gap-1 rounded-lg border px-1.5 py-1"
                  >
                    <Badge variant="outline" className={roleBadgeClass(role.name)}>
                      {titleCase(role.name)}
                    </Badge>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      className="rounded-md text-destructive hover:bg-destructive/10 hover:text-destructive"
                      aria-label={`Revoke ${titleCase(role.name)} from ${user.name}`}
                      disabled={revokeMutation.isPending}
                      onClick={() => revokeMutation.mutate(role)}
                    >
                      <XIcon />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">No roles assigned.</p>
            )}
          </div>

          <div className="border-t" />

          <div className="space-y-2">
            <label htmlFor="user-role" className="text-sm font-medium">
              Assign another role
            </label>
            <Select value={roleId} onValueChange={(value) => setRoleId(value ?? "")}>
              <SelectTrigger id="user-role" className="w-full rounded-lg bg-background">
                <SelectValue placeholder={isLoading ? "Loading roles..." : "Select a role"}>
                  {selectedRole ? titleCase(selectedRole.name) : undefined}
                </SelectValue>
              </SelectTrigger>
              <SelectContent className="rounded-lg">
                {assignableRoles.map((role) => (
                  <SelectItem key={role.pid} value={String(role.id)}>
                    <span>{titleCase(role.name)}</span>
                    {role.description && (
                      <span className="text-xs font-normal text-muted-foreground">
                        {role.description}
                      </span>
                    )}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {!isLoading && !assignableRoles.length && (
              <p className="text-sm text-muted-foreground">
                This user already has every available role.
              </p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            className="rounded-lg"
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            type="button"
            className="rounded-lg"
            disabled={!roleId || mutation.isPending}
            onClick={() => mutation.mutate()}
          >
            <UserPlusIcon />
            {mutation.isPending ? "Assigning..." : "Assign role"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
