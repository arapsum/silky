use std::{borrow::Cow, sync::LazyLock};

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub static RE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9- ]+$").expect("Regex initialisation failed"));

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewCategory<'a> {
    #[validate(custom(function = "validate_name"))]
    name: Cow<'a, str>,
    #[validate(url)]
    image_link: Cow<'a, str>,
    #[validate(range(min = 1, max = 1_000_000))]
    parent_id: Option<i32>,
    #[validate(length(max = 1000, message = "Description must be under 1000 characters"))]
    description: Option<Cow<'a, str>>,
}

impl<'a> NewCategory<'a> {
    #[must_use]
    pub const fn new(
        name: Cow<'a, str>,
        image_link: Cow<'a, str>,
        parent_id: Option<i32>,
        description: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            name,
            image_link,
            parent_id,
            description,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn image_link(&self) -> &str {
        &self.image_link
    }

    #[must_use]
    pub const fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Cow<'a, str>> {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategory<'a> {
    #[validate(custom(function = "validate_name"))]
    name: Option<Cow<'a, str>>,
    #[validate(url)]
    image_link: Option<Cow<'a, str>>,
    #[validate(range(min = 1, max = 1_000_000))]
    parent_id: Option<i32>,
    #[validate(length(max = 1000, message = "Description must be under 1000 characters"))]
    description: Option<Cow<'a, str>>,
}

impl<'a> UpdateCategory<'a> {
    #[must_use]
    pub const fn new(
        name: Option<Cow<'a, str>>,
        image_link: Option<Cow<'a, str>>,
        parent_id: Option<i32>,
        description: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            name,
            image_link,
            parent_id,
            description,
        }
    }
    #[must_use]
    pub const fn name(&self) -> Option<&Cow<'a, str>> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn image_link(&self) -> Option<&Cow<'a, str>> {
        self.image_link.as_ref()
    }

    #[must_use]
    pub const fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    #[must_use]
    pub const fn description(&self) -> Option<&Cow<'a, str>> {
        self.description.as_ref()
    }
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 2;
    const MAX_LENGTH: usize = 48;

    let name = name.trim();
    let length = name.len();

    let error: ValidationError;

    if name.is_empty() {
        error = ValidationError::new("empty_name");
        return Err(error.with_message(Cow::Borrowed("Name is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_name");
        return Err(error.with_message(Cow::Borrowed("Name requires 2 characters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_name");
        return Err(error.with_message(Cow::Borrowed("Name must be under 48 characters")));
    }

    RE_NAME.captures(name).map_or_else(
        || {
            let val_error = ValidationError::new("invalid_name");
            Err(val_error.with_message(Cow::Borrowed(
                "Only letters, numbers and hyphens can be used.",
            )))
        },
        |_captures: Captures<'_>| Ok(()),
    )
}
