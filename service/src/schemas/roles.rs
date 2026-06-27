use std::{borrow::Cow, sync::LazyLock};

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub static RE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_ ]+$").expect("Regex initialisation failed"));

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct NewRole<'a> {
    #[validate(custom(function = "validate_name"))]
    pub(crate) name: Cow<'a, str>,

    #[validate(length(max = 256, message = "Description must be under 256 characters"))]
    pub(crate) description: Option<Cow<'a, str>>,
}

impl<'a> NewRole<'a> {
    #[must_use]
    pub const fn new(name: Cow<'a, str>, description: Option<Cow<'a, str>>) -> Self {
        Self { name, description }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Cow<'a, str>> {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct UpdateRole<'a> {
    #[validate(custom(function = "validate_name"))]
    pub(crate) name: Option<Cow<'a, str>>,

    #[validate(length(max = 256, message = "Description must be under 256 characters"))]
    pub(crate) description: Option<Cow<'a, str>>,
}

impl<'a> UpdateRole<'a> {
    #[must_use]
    pub const fn new(name: Option<Cow<'a, str>>, description: Option<Cow<'a, str>>) -> Self {
        Self { name, description }
    }

    #[must_use]
    pub const fn name(&self) -> Option<&Cow<'a, str>> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Cow<'a, str>> {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AssignRole {
    #[validate(range(min = 1, message = "User ID must be a positive integer"))]
    user_id: i32,

    #[validate(range(min = 1, message = "Role ID must be a positive integer"))]
    role_id: i32,
}

impl AssignRole {
    #[must_use]
    pub const fn user_id(&self) -> i32 {
        self.user_id
    }

    #[must_use]
    pub const fn role_id(&self) -> i32 {
        self.role_id
    }
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 2;
    const MAX_LENGTH: usize = 32;

    let name = name.trim();
    let length = name.len();

    let error: ValidationError;

    if name.is_empty() {
        error = ValidationError::new("empty_name");
        return Err(error.with_message(Cow::Borrowed("Name is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_name");
        return Err(error.with_message(Cow::Borrowed("Name requires 2 letters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_name");
        return Err(error.with_message(Cow::Borrowed("Name must be under 32 letters")));
    }

    RE_NAME.captures(name).map_or_else(
        || {
            let val_error = ValidationError::new("invalid_name");
            Err(val_error.with_message(Cow::Borrowed(
                "Only letters, numbers and underscores can be used.",
            )))
        },
        |_captures: Captures<'_>| Ok(()),
    )
}
