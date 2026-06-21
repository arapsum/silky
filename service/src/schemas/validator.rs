use std::{borrow::Cow, collections::BTreeMap};

use validator::Validate;

use crate::{Error, Result};

pub struct Validator<T>(pub T)
where
    T: Validate;

impl<T> Validator<T>
where
    T: Validate,
{
    pub const fn new(t: T) -> Self {
        Self(t)
    }

    /// Validates the wrapped value and returns a reference to it on success.
    ///
    /// This method delegates validation to the underlying [`Validate`]
    /// implementation and converts any validation failures into
    /// [`Error::ValidationError`].
    ///
    /// When validation fails, all field-level validation errors are collected
    /// into a JSON object where each key is the field name and each value is a
    /// comma-separated string containing the corresponding validation messages.
    ///
    /// # Returns
    ///
    /// - `Ok(&T)` if all validation rules pass.
    /// - `Err(Error::ValidationError)` if one or more validation rules fail.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ValidationError`] when validation of the wrapped value
    /// fails. This can occur when:
    ///
    /// - A validation constraint defined on a field is not satisfied.
    /// - A custom validator returns an error.
    /// - Validation infrastructure used by the underlying [`validator`] crate
    ///   reports an error (for example, an invalid regular expression used by a
    ///   validation rule).
    pub fn validate(&self) -> Result<&T> {
        match self.0.validate() {
            Ok(()) => Ok(&self.0),
            Err(val_errors) => {
                let mut errors: BTreeMap<String, String> = BTreeMap::new();

                val_errors.field_errors().into_iter().for_each(
                    |(key, value): (Cow<'static, str>, &Vec<validator::ValidationError>)| {
                        errors.insert(
                            key.to_string(),
                            value
                                .iter()
                                .map(|err| err.message.as_deref().unwrap_or("Field error"))
                                .collect::<Vec<&str>>()
                                .join(", "),
                        );
                    },
                );

                Err(Error::ValidationError(serde_json::json!(errors).to_string()).into())
            }
        }
    }
}
