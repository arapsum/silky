import { apiRequest } from "#/api/client.ts";

export const rolesQueryKey = ["roles"] as const;

export type RoleUser = {
  pid: string;
  name: string;
  email: string;
  image: string | null;
};

export type Role = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
  users: RoleUser[];
  createdAt: string;
  updatedAt: string;
};

export type RoleInput = {
  name: string;
  description?: string;
};

export type AssignPermissionInput = {
  roleId: number;
  permissionId: number;
};

export type RolePermission = {
  id: number;
  pid: string;
  roleId: number;
  permissionId: number;
  createdAt: string;
  updatedAt: string;
};

export function listRoles() {
  return apiRequest<Role[]>("/roles", {
    fallback: "Unable to load roles",
  });
}

export function createRole(input: RoleInput) {
  return apiRequest<Role>("/roles", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to create role",
  });
}

export function updateRole(pid: string, input: RoleInput) {
  return apiRequest<Role>(`/roles/${pid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update role",
  });
}

export function assignPermissionToRole(input: AssignPermissionInput) {
  return apiRequest<RolePermission>("/roles/permissions", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to assign permission",
  });
}
