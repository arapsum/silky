use serde_json::json;
use serial_test::serial;
use service::models::{FinalizeMediaAsset, MediaAsset, NewPicture, Picture};
use uuid::Uuid;

use crate::{boot_test, seed_data};

const JOHN_DOE_PID: &str = "bd6f7c26-d2c9-487e-b837-8f77be468033";

fn finalize_input(public_id: String) -> FinalizeMediaAsset {
    FinalizeMediaAsset {
        public_id: public_id.clone(),
        asset_id: Some("cloudinary-asset-id".to_string()),
        secure_url: format!("https://res.cloudinary.com/test-cloud/image/upload/{public_id}.png"),
        format: Some("png".to_string()),
        bytes: Some(1024),
        width: Some(640),
        height: Some(480),
        checksum: Some("checksum".to_string()),
    }
}

#[tokio::test]
#[serial]
async fn media_asset_lifecycle_finalizes_normalized_public_ids_and_quarantines_orphans() {
    let ctx = boot_test().await.expect("Failed to boot test");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let asset = MediaAsset::create_pending(
        ctx.db(),
        Uuid::parse_str(JOHN_DOE_PID).unwrap(),
        "product-image",
        "silk/products",
        Some("checksum"),
    )
    .await
    .expect("Failed to create pending asset");

    let finalized = MediaAsset::finalize(
        ctx.db(),
        asset.pid(),
        &finalize_input("silk/products/product-image".to_string()),
    )
    .await
    .expect("Failed to finalize asset");

    let finalized_json = serde_json::to_value(&finalized).unwrap();
    assert_eq!(finalized_json["status"], json!("active"));
    assert_eq!(finalized.public_id(), "silk/products/product-image");

    let quarantined = MediaAsset::quarantine_orphans(ctx.db())
        .await
        .expect("Failed to quarantine orphaned assets");
    let quarantined_asset = quarantined
        .iter()
        .find(|candidate| candidate.pid() == asset.pid())
        .expect("Expected finalized asset to be quarantined");

    assert_eq!(
        serde_json::to_value(quarantined_asset).unwrap()["status"],
        json!("quarantined")
    );
}

#[tokio::test]
#[serial]
async fn referenced_media_asset_is_not_quarantined() {
    let ctx = boot_test().await.expect("Failed to boot test");
    seed_data(ctx.db()).await.expect("Failed to seed data");

    let asset = MediaAsset::create_pending(
        ctx.db(),
        Uuid::parse_str(JOHN_DOE_PID).unwrap(),
        "referenced-product-image",
        "silk/products",
        None,
    )
    .await
    .expect("Failed to create pending asset");
    let finalized = MediaAsset::finalize(
        ctx.db(),
        asset.pid(),
        &finalize_input("silk/products/referenced-product-image".to_string()),
    )
    .await
    .expect("Failed to finalize asset");

    let picture = NewPicture::new(203, finalized.secure_url().to_string(), None, Some(99))
        .with_media_asset_pid(Some(finalized.pid()));
    Picture::create(ctx.db(), &picture)
        .await
        .expect("Failed to link media asset to product picture");

    let quarantined = MediaAsset::quarantine_orphans(ctx.db())
        .await
        .expect("Failed to quarantine orphaned assets");

    assert!(
        !quarantined
            .iter()
            .any(|candidate| candidate.pid() == finalized.pid())
    );
    let stored = MediaAsset::find_by_pid(ctx.db(), finalized.pid())
        .await
        .expect("Referenced asset should still exist");
    assert_eq!(
        serde_json::to_value(stored).unwrap()["status"],
        json!("active")
    );
}
