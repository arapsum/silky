import { useQuery } from "@tanstack/react-query";
import { currentUserQueryKey, getCurrentUser } from "#/api/account.ts";
import {
  hasAnyPermission,
  hasEveryPermission,
  hasPermission,
  type PermissionName,
} from "#/lib/access.ts";

export function useAccess() {
  const currentUserQuery = useQuery({
    queryKey: currentUserQueryKey,
    queryFn: getCurrentUser,
    retry: false,
  });
  const currentUser = currentUserQuery.data;

  return {
    currentUser,
    isLoading: currentUserQuery.isLoading,
    can: (permission: PermissionName) => hasPermission(currentUser, permission),
    canEvery: (permissions: readonly PermissionName[]) =>
      hasEveryPermission(currentUser, permissions),
    canAny: (permissions: readonly PermissionName[]) => hasAnyPermission(currentUser, permissions),
  };
}
