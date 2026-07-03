use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, prelude::FromRow};
use uuid::Uuid;

use crate::models::Category;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub image_link: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub parent_name: Option<String>,
    pub product_count: i32,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

impl CategoryResponse {
    #[must_use]
    pub fn new(category: &Category, product_count: i32, parent_name: Option<&str>) -> Self {
        Self {
            id: category.id(),
            pid: category.pid(),
            name: category.name().to_string(),
            image_link: category.image_link().to_string(),
            description: category.description().map(|d| d.to_string()),
            parent_id: category.parent_id(),
            parent_name: parent_name.map(|n| n.to_string()),
            product_count,
            created_at: category.created_at(),
            updated_at: category.updated_at(),
            deleted_at: category.deleted_at().clone(),
        }
    }
}
