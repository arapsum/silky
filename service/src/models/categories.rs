use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pid: Uuid,
    name: String,
    image_link: String,
    description: Option<String>,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    deleted_at: Option<DateTime<FixedOffset>>,
}

impl Category {
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    #[must_use]
    pub const fn pid(&self) -> Uuid {
        self.pid
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
    pub const fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    #[must_use]
    pub const fn deleted_at(&self) -> Option<DateTime<FixedOffset>> {
        self.deleted_at
    }
}
