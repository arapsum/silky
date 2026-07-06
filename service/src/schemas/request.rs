use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListQuery {
    #[validate(range(min = 1, message = "Limit must be a positive integer"))]
    limit: Option<i64>,
    #[validate(range(min = 1, message = "Page must be a positive integer"))]
    page: Option<i64>,
    search: Option<String>,
    name: Option<String>,
    slug: Option<String>,
    #[validate(range(min = 1, message = "Parent ID must be a positive integer"))]
    parent_id: Option<i32>,
    has_parent: Option<bool>,
    include_deleted: Option<bool>,
}

impl CategoryListQuery {
    #[must_use]
    pub const fn limit(&self) -> Option<i64> {
        self.limit
    }

    #[must_use]
    pub const fn page(&self) -> Option<i64> {
        self.page
    }

    #[must_use]
    pub fn search(&self) -> Option<&str> {
        self.search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    #[must_use]
    pub fn slug(&self) -> Option<&str> {
        self.slug
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    #[must_use]
    pub const fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    #[must_use]
    pub const fn has_parent(&self) -> Option<bool> {
        self.has_parent
    }

    #[must_use]
    pub const fn include_deleted(&self) -> bool {
        matches!(self.include_deleted, Some(true))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "Limit must be a positive integer"))]
    limit: Option<i64>,
    #[validate(range(min = 1, message = "Page must be a positive integer"))]
    page: Option<i64>,
}

impl PaginationQuery {
    #[must_use]
    pub const fn limit(&self) -> Option<i64> {
        self.limit
    }

    #[must_use]
    pub const fn page(&self) -> Option<i64> {
        self.page
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct PermissionRoleQuery {
    #[validate(range(min = 1, message = "Role ID must be a positive integer"))]
    role_id: Option<i32>,
    #[validate(range(min = 1, message = "Permission ID must be a positive integer"))]
    permission_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct PermissionListQuery {
    role: Option<String>,
}

impl PermissionListQuery {
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct UserListQuery {
    role: Option<String>,
}

impl UserListQuery {
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}

impl PermissionRoleQuery {
    #[must_use]
    pub const fn role_id(&self) -> Option<i32> {
        self.role_id
    }

    #[must_use]
    pub const fn permission_id(&self) -> Option<i32> {
        self.permission_id
    }
}
