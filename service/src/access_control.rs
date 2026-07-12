#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PermissionName(&'static str);

impl PermissionName {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl AsRef<str> for PermissionName {
    fn as_ref(&self) -> &str {
        self.0
    }
}

pub mod permissions {
    use super::PermissionName;

    pub mod roles {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("roles:read");
        pub const CREATE: PermissionName = PermissionName("roles:create");
        pub const UPDATE: PermissionName = PermissionName("roles:update");
        pub const DELETE: PermissionName = PermissionName("roles:delete");
    }

    pub mod permission_records {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("permissions:read");
        pub const CREATE: PermissionName = PermissionName("permissions:create");
        pub const UPDATE: PermissionName = PermissionName("permissions:update");
        pub const DELETE: PermissionName = PermissionName("permissions:delete");
    }

    pub mod users {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("users:read");
        pub const CREATE: PermissionName = PermissionName("users:create");
        pub const UPDATE: PermissionName = PermissionName("users:update");
        pub const DELETE: PermissionName = PermissionName("users:delete");
    }

    pub mod categories {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("categories:read");
        pub const CREATE: PermissionName = PermissionName("categories:create");
        pub const UPDATE: PermissionName = PermissionName("categories:update");
        pub const DELETE: PermissionName = PermissionName("categories:delete");
    }

    pub mod products {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("products:read");
        pub const CREATE: PermissionName = PermissionName("products:create");
        pub const UPDATE: PermissionName = PermissionName("products:update");
        pub const DELETE: PermissionName = PermissionName("products:delete");
    }

    pub mod media {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("media:read");
        pub const CREATE: PermissionName = PermissionName("media:create");
        pub const UPDATE: PermissionName = PermissionName("media:update");
        pub const DELETE: PermissionName = PermissionName("media:delete");
    }

    pub mod orders {
        use super::PermissionName;

        pub const READ: PermissionName = PermissionName("orders:read");
        pub const UPDATE: PermissionName = PermissionName("orders:update");
    }
}
