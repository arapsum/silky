use serial_test::serial;
use service::models::{ModelError, Tag};
use uuid::Uuid;

use crate::{boot_test, seed_data};

const PRODUCT_PID: &str = "6d7b16c3-efbf-4e7e-9b70-4f43e1cc3001";

#[tokio::test]
#[serial]
async fn can_manage_product_tags() {
    let ctx = boot_test().await.expect("Failed to boot test");
    seed_data(ctx.db()).await.expect("Failed to seed data");
    sqlx::query(
        "DELETE FROM tags WHERE name IN ('Seasonal test', 'Seasonal test updated', 'Featured test')",
    )
        .execute(ctx.db())
        .await
        .expect("Failed to clean product tags");

    let seasonal = Tag::create(ctx.db(), "  Seasonal test  ")
        .await
        .expect("Failed to create product tag");
    assert_eq!(seasonal.name(), "Seasonal test");

    let duplicate = Tag::create(ctx.db(), "SEASONAL TEST").await;
    assert!(matches!(duplicate, Err(ModelError::EntityAlreadyExists(_))));

    let found = Tag::find_by_pid(ctx.db(), seasonal.pid())
        .await
        .expect("Failed to find product tag by PID");
    assert_eq!(found.name(), seasonal.name());

    let seasonal = Tag::update(ctx.db(), seasonal.pid(), " Seasonal test updated ")
        .await
        .expect("Failed to update product tag");
    let found = Tag::find_by_name(ctx.db(), " seasonal TEST UPDATED ")
        .await
        .expect("Failed to find product tag by name");
    assert_eq!(found.pid(), seasonal.pid());

    let featured = Tag::create(ctx.db(), "Featured test")
        .await
        .expect("Failed to create second product tag");
    assert!(
        Tag::find_all(ctx.db())
            .await
            .expect("Failed to list product tags")
            .iter()
            .any(|tag| tag.pid() == featured.pid())
    );
    let product_pid = Uuid::parse_str(PRODUCT_PID).expect("Failed to parse product PID");

    Tag::add_to_product(ctx.db(), product_pid, seasonal.pid())
        .await
        .expect("Failed to assign product tag");
    Tag::add_to_product(ctx.db(), product_pid, seasonal.pid())
        .await
        .expect("Failed to make product tag assignment idempotent");
    assert_eq!(
        Tag::find_by_product(ctx.db(), product_pid)
            .await
            .expect("Failed to list product tags")
            .iter()
            .filter(|tag| tag.pid() == seasonal.pid())
            .count(),
        1
    );

    let tags = Tag::set_for_product(ctx.db(), product_pid, &[featured.pid(), featured.pid()])
        .await
        .expect("Failed to replace product tags");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].pid(), featured.pid());

    assert!(
        Tag::remove_from_product(ctx.db(), product_pid, featured.pid())
            .await
            .expect("Failed to remove product tag")
    );
    assert!(
        Tag::find_by_product(ctx.db(), product_pid)
            .await
            .expect("Failed to list cleared product tags")
            .is_empty()
    );

    Tag::delete(ctx.db(), seasonal.pid())
        .await
        .expect("Failed to delete first product tag");
    Tag::delete(ctx.db(), featured.pid())
        .await
        .expect("Failed to delete second product tag");
}
