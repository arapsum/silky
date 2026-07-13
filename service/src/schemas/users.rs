use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::auth::{validate_name, validate_password};

/// The details an authorised staff member supplies when creating another
/// staff account.
///
/// A role is required at creation time. The repository rejects the customer
/// role so this payload cannot be used to provision storefront customers.
#[derive(Debug, Deserialize, Clone, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateStaffUser<'a> {
    #[validate(email(message = "Invalid email address"))]
    email: Cow<'a, str>,
    #[validate(custom(function = "validate_name"))]
    name: Cow<'a, str>,
    #[validate(custom(function = "validate_password"))]
    password: Cow<'a, str>,
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    confirm_password: Cow<'a, str>,
    #[validate(range(min = 1, message = "A staff role is required"))]
    role_id: i32,
    #[validate(url(message = "Invalid image URL"))]
    image: Option<Cow<'a, str>>,
}

impl<'a> CreateStaffUser<'a> {
    #[must_use]
    pub const fn new(
        email: Cow<'a, str>,
        name: Cow<'a, str>,
        password: Cow<'a, str>,
        confirm_password: Cow<'a, str>,
        role_id: i32,
        image: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            email,
            name,
            password,
            confirm_password,
            role_id,
            image,
        }
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub const fn role_id(&self) -> i32 {
        self.role_id
    }

    #[must_use]
    pub const fn image(&self) -> Option<&Cow<'a, str>> {
        self.image.as_ref()
    }
}
