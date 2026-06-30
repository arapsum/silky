use serde::{Deserialize, Serialize};
use validator::Validate;

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
