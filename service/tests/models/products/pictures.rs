use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::{NewPicture, Picture};

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("pictures");
        settings.set_snapshot_path("snapshots/pictures");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case(
    "can_create_product_picture_successful",
    203,
    None,
    "https://cdn.example.com/products/leather-bag-main.png",
    Some(3)
)]
#[case(
    "can_create_variant_picture_successful",
    203,
    Some(205),
    "https://cdn.example.com/products/court-leather-sneakers-alt.png",
    Some(2)
)]
#[case(
    "can_create_picture_without_display_order",
    203,
    None,
    "https://cdn.example.com/products/leather-bag-detail.png",
    None
)]
#[case(
    "cannot_create_picture_when_product_does_not_exist",
    999,
    None,
    "https://cdn.example.com/products/missing-product.png",
    Some(1)
)]
#[case(
    "cannot_create_picture_when_variant_does_not_exist",
    203,
    Some(999),
    "https://cdn.example.com/products/missing-variant.png",
    Some(1)
)]
#[tokio::test]
#[serial]
async fn can_create_picture(
    #[case] test_name: &str,
    #[case] product_id: i32,
    #[case] variant_id: Option<i32>,
    #[case] image_link: &str,
    #[case] display_order: Option<i32>,
) {
    configure_insta!();

    let ctx = boot_test().await.expect("Failed to boot test!");

    seed_data(ctx.db()).await.expect("Failed to seed data");

    let params = NewPicture::new(
        product_id,
        image_link.to_string(),
        variant_id,
        display_order,
    );

    let result = Picture::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id().to_vec());
            filters
        }
    }, {
            assert_debug_snapshot!(test_name, result)
    });
}
