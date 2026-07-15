pub mod addresses;
pub mod cart;
pub mod categories;
pub mod error;
pub mod media_assets;
pub mod order_items;
pub mod orders;
pub mod payment_attempts;
pub mod permissions;
pub mod products;
pub mod roles;
pub mod roles_permissions;
pub mod seed;
pub mod stripe_webhook_events;
pub mod user;
pub mod user_roles;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub use self::{
    addresses::Address,
    cart::CartQuote,
    categories::{Category, CategoryAttributeLink},
    error::{ModelError, ModelResult},
    media_assets::{FinalizeMediaAsset, MediaAsset},
    order_items::OrderItem,
    orders::{CheckoutOrderState, Order, OrderWithItems},
    payment_attempts::{CheckoutSessionDetails, PaymentAttempt, PaymentReconciliation},
    permissions::Permission,
    products::{
        Attribute, AttributeValue, NewPicture, NewProductOption, NewVariant,
        NewVariantAttributeValue, Picture, Product, ProductOption, ProductVariant, Tag,
        VariantAttributeValue,
    },
    roles::Role,
    roles_permissions::RolePermission,
    seed::Seedable,
    stripe_webhook_events::StripeWebhookEvent,
    user::{User, UserAccess},
    user_roles::UserRole,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    page: i64,
    limit: i64,
    total_items: i64,
    total_pages: i64,
    has_next: bool,
    has_prev: bool,
}

impl Pagination {
    #[must_use]
    pub fn new(page: i64, limit: i64, total_items: i64) -> Self {
        let limit = limit.max(1);
        let total_pages = if total_items == 0 {
            0
        } else {
            (total_items + limit - 1) / limit
        };

        Self {
            page,
            limit,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PaginatedModel<T>
where
    T: Debug + Clone + Serialize,
{
    data: Vec<T>,
    pagination: Pagination,
}

impl<T> PaginatedModel<T>
where
    T: Debug + Serialize + Clone,
{
    #[must_use]
    pub const fn new(data: Vec<T>, pagination: Pagination) -> Self {
        Self { data, pagination }
    }
}
