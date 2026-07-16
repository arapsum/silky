use serde::{Deserialize, Serialize};
use validator::Validate;

use super::NewOrderItem;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CartQuoteRequest {
    #[validate(length(min = 1, max = 100), nested)]
    items: Vec<NewOrderItem>,
}

impl CartQuoteRequest {
    #[must_use]
    pub fn items(&self) -> &[NewOrderItem] {
        &self.items
    }
}
