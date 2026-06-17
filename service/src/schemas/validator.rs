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

    /// Returns the validate of this [`Validator<T>`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    ///  Specified validation constraint failed
    ///  Regex does not get instantiated
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

                Err(Error::ValidationError(
                    serde_json::json!(errors).to_string(),
                ))
            }
        }
    }
}
