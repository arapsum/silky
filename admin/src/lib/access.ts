export const PERMISSIONS = {
  admin: {
    access: "admin:access",
  },
  roles: {
    read: "roles:read",
    create: "roles:create",
    update: "roles:update",
    delete: "roles:delete",
  },
  permissions: {
    read: "permissions:read",
    create: "permissions:create",
    update: "permissions:update",
    delete: "permissions:delete",
  },
  users: {
    read: "users:read",
    create: "users:create",
    update: "users:update",
    delete: "users:delete",
  },
  categories: {
    read: "categories:read",
    create: "categories:create",
    update: "categories:update",
    delete: "categories:delete",
  },
  products: {
    read: "products:read",
    create: "products:create",
    update: "products:update",
    delete: "products:delete",
  },
  media: {
    read: "media:read",
    create: "media:create",
    update: "media:update",
    delete: "media:delete",
  },
  orders: {
    read: "orders:read",
    update: "orders:update",
  },
} as const;

type NestedValue<T> = T extends string ? T : { [Key in keyof T]: NestedValue<T[Key]> }[keyof T];

export type PermissionName = NestedValue<typeof PERMISSIONS>;

export type AccessSubject = {
  roles: readonly string[];
  permissions: readonly string[];
};

export function hasPermission(
  subject: AccessSubject | null | undefined,
  permission: PermissionName,
) {
  return subject?.permissions.includes(permission) ?? false;
}

export function hasEveryPermission(
  subject: AccessSubject | null | undefined,
  permissions: readonly PermissionName[],
) {
  return permissions.every((permission) => hasPermission(subject, permission));
}

export function hasAnyPermission(
  subject: AccessSubject | null | undefined,
  permissions: readonly PermissionName[],
) {
  return permissions.some((permission) => hasPermission(subject, permission));
}
