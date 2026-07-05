import { apiRequest } from "#/api/client.ts";

export const usersQueryKey = ["users"] as const;

export type UserRole = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
};

export type User = {
  id: number;
  pid: string;
  name: string;
  email: string;
  image: string | null;
  verified: boolean;
  roles: UserRole[];
  createdAt: string;
  updatedAt: string;
};

export function listUsers(role?: string) {
  const params = new URLSearchParams();

  if (role) {
    params.set("role", role);
  }

  const query = params.toString();

  return apiRequest<User[]>(`/users${query ? `?${query}` : ""}`, {
    fallback: "Unable to load users",
  });
}
