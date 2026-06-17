use std::{borrow::Cow, sync::LazyLock};

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub static RE_USERNAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").expect("Regex initialisation failed"));

pub static RE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$",
    )
    .expect("Regex initialisation failed")
});

#[derive(Debug, Deserialize, Clone, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterUser<'a> {
    #[validate(email(message = "Invalid email address"))]
    email: Cow<'a, str>,
    #[validate(custom(function = "validate_username"))]
    username: Cow<'a, str>,
    #[validate(custom(function = "validate_password"))]
    password: Cow<'a, str>,
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    confirm_password: Cow<'a, str>,
}

impl<'a> RegisterUser<'a> {
    #[must_use]
    pub const fn new(
        email: Cow<'a, str>,
        username: Cow<'a, str>,
        password: Cow<'a, str>,
        confirm_password: Cow<'a, str>,
    ) -> Self {
        Self {
            email,
            username,
            password,
            confirm_password,
        }
    }

    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub fn confirm_password(&self) -> &str {
        &self.confirm_password
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, Validate)]
pub struct LoginUser<'a> {
    #[validate(email(message = "Invalid email address"))]
    email: Cow<'a, str>,

    #[validate(length(min = 8, message = "Password must be atleast 8 characters long"))]
    password: Cow<'a, str>,
}

impl LoginUser<'_> {
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ForgotPassword<'a> {
    #[validate(email(message = "Invalid email address"))]
    email: Cow<'a, str>,
}

impl<'a> ForgotPassword<'a> {
    #[must_use]
    pub const fn new(email: Cow<'a, str>) -> Self {
        Self { email }
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Validate, Clone, Deserialize)]
pub struct ResetPassword<'a> {
    #[validate(custom(function = "validate_token"))]
    token: Cow<'a, str>,
    #[validate(custom(function = "validate_password"))]
    password: Cow<'a, str>,
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    confirm_password: Cow<'a, str>,
}

impl<'a> ResetPassword<'a> {
    #[must_use]
    pub const fn new(
        token: Cow<'a, str>,
        password: Cow<'a, str>,
        confirm_password: Cow<'a, str>,
    ) -> Self {
        Self {
            token,
            password,
            confirm_password,
        }
    }

    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub fn confirm_password(&self) -> &str {
        &self.confirm_password
    }
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 8;
    const MAX_LENGTH: usize = 48;

    let password = password.trim();
    let length = password.len();

    let error: ValidationError;

    if password.is_empty() {
        error = ValidationError::new("empty_password");
        return Err(error.with_message(Cow::Borrowed("password is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_password");
        return Err(error.with_message(Cow::Borrowed("password requires 8 characters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_password");
        return Err(error.with_message(Cow::Borrowed("password must be under 48 characters")));
    }

    if password.contains(char::is_whitespace) {
        error = ValidationError::new("whitespace_in_password");
        return Err(error.with_message(Cow::Borrowed("password cannot have spaces")));
    }

    if password.contains(',') {
        error = ValidationError::new("commas_in_password");
        return Err(error.with_message(Cow::Borrowed("password cannot have commas")));
    }

    Ok(())
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 6;
    const MAX_LENGTH: usize = 32;

    let username = username.trim();
    let length = username.len();

    let error: ValidationError;

    if username.is_empty() {
        error = ValidationError::new("empty_username");
        return Err(error.with_message(Cow::Borrowed("Username is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_username");
        return Err(error.with_message(Cow::Borrowed("Username requires 6 letters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_username");
        return Err(error.with_message(Cow::Borrowed("Username must be under 32 letters")));
    }

    RE_USERNAME.captures(username).map_or_else(
        || {
            let val_error = ValidationError::new("invalid_name");
            Err(val_error.with_message(Cow::Borrowed(
                "Only letters, numbers and underscores can be used.",
            )))
        },
        |_captures: Captures<'_>| Ok(()),
    )
}

fn validate_token(token: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 32;
    const MAX_LENGTH: usize = 40;

    let token = token.trim();
    let length = token.len();

    let error: ValidationError;

    if token.is_empty() {
        error = ValidationError::new("empty_token");
        return Err(error.with_message(Cow::Borrowed("Token is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_token");
        return Err(error.with_message(Cow::Borrowed("Token is invalid")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_token");
        return Err(error.with_message(Cow::Borrowed("Token is invalid")));
    }

    RE_TOKEN.captures(token).map_or_else(
        || {
            let val_error = ValidationError::new("invalid_token");
            Err(val_error.with_message(Cow::Borrowed("Inalid token.")))
        },
        |_captures: Captures<'_>| Ok(()),
    )
}
