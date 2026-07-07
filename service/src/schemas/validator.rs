use std::collections::BTreeMap;

use validator::{Validate, ValidationErrors, ValidationErrorsKind};

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

                collect_errors("", &val_errors, &mut errors);

                Err(Error::ValidationError(serde_json::json!(errors).to_string()).into())
            }
        }
    }
}

fn collect_errors(prefix: &str, errors: &ValidationErrors, output: &mut BTreeMap<String, String>) {
    for (field, kind) in errors.errors() {
        let key = join_key(prefix, field.as_ref());

        match kind {
            ValidationErrorsKind::Field(field_errors) => {
                output.insert(
                    key,
                    field_errors
                        .iter()
                        .map(|err| err.message.as_deref().unwrap_or("Field error"))
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
            }
            ValidationErrorsKind::Struct(nested_errors) => {
                collect_errors(&key, nested_errors, output);
            }
            ValidationErrorsKind::List(nested_errors) => {
                for (index, nested_errors) in nested_errors {
                    collect_errors(&format!("{key}[{index}]"), nested_errors, output);
                }
            }
        }
    }
}

fn join_key(prefix: &str, field: &str) -> String {
    if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{prefix}.{field}")
    }
}
