import { apiRequest } from "#/api/client.ts";

export const permissionsQueryKey = ["permissions"] as const;

export type Permission = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
  createdAt: string;
  updatedAt: string;
};

export function listPermissions(role?: string) {
  const params = new URLSearchParams();

  if (role) {
    params.set("role", role);
  }

  const query = params.toString();

  return apiRequest<Permission[]>(`/permissions${query ? `?${query}` : ""}`, {
    fallback: "Unable to load permissions",
  });
}

export function getPermission(pid: string) {
  return apiRequest<Permission>(`/permissions/${pid}`, {
    fallback: "Unable to load permission",
  });
}
