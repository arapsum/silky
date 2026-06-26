use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serde_json::{Value, json};
use serial_test::serial;
use service::{
    models::Category,
    schemas::{NewCategory, PaginationQuery, UpdateCategory},
};
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("categories");
        settings.set_snapshot_path("snapshots/categories");
        let _guard = settings.bind_to_scope();
    };
}

fn new_category(
    name: String,
    image_link: String,
    parent_id: Option<i32>,
    description: Option<String>,
) -> NewCategory<'static> {
    NewCategory::new(
        Cow::Owned(name),
        Cow::Owned(image_link),
        parent_id,
        description.map(Cow::Owned),
    )
}

fn update_category(
    name: Option<String>,
    image_link: Option<String>,
    parent_id: Option<i32>,
    description: Option<String>,
) -> UpdateCategory<'static> {
    UpdateCategory::new(
        name.map(Cow::Owned),
        image_link.map(Cow::Owned),
        parent_id,
        description.map(Cow::Owned),
    )
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

fn pagination_query(limit: Option<i64>, page: Option<i64>) -> PaginationQuery {
    let mut value = serde_json::Map::new();

    if let Some(limit) = limit {
        value.insert("limit".to_string(), json!(limit));
    }

    if let Some(page) = page {
        value.insert("page".to_string(), json!(page));
    }

    serde_json::from_value(Value::Object(value)).expect("Failed to parse pagination query")
}

#[rstest]
#[case(
    "can_create_category_with_description",
    "Accessories".to_string(),
    "https://cdn.example.com/categories/accessories.png".to_string(),
    Some("Bags and belts".to_string())
)]
#[case(
    "can_create_category_without_description",
    "Hats".to_string(),
    "https://cdn.example.com/categories/hats.png".to_string(),
    None
)]
#[case(
    "can_create_category_and_normalize_name",
    "  Winter Wear  ".to_string(),
    "  https://cdn.example.com/categories/winter.png  ".to_string(),
    Some("Warm layers".to_string())
)]
#[tokio::test]
#[serial]
async fn can_create_category(
    #[case] test_name: &str,
    #[case] name: String,
    #[case] image_link: String,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();
    let params = new_category(name, image_link, None, description);

    let result = Category::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case(
    "cannot_create_category_when_name_already_exists",
    "Accessories".to_string(),
    "accessories".to_string()
)]
#[case(
    "cannot_create_category_when_name_differs_by_case_and_whitespace",
    "  Accessories  ".to_string(),
    "ACCESSORIES".to_string()
)]
#[tokio::test]
#[serial]
async fn cannot_create_duplicate_category(
    #[case] test_name: &str,
    #[case] first_name: String,
    #[case] second_name: String,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let first = new_category(
        first_name,
        "https://cdn.example.com/categories/accessories.png".to_string(),
        None,
        Some("Bags and belts".to_string()),
    );
    Category::create(ctx.db(), &first).await.unwrap();

    let second = new_category(
        second_name,
        "https://cdn.example.com/categories/duplicate.png".to_string(),
        None,
        Some("Duplicate category".to_string()),
    );
    let result = Category::create(ctx.db(), &second).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_seed_categories_from_json", "categories.json")]
#[case(
    "when_seed_file_does_not_exist_category_seeding_fails",
    "missing-categories.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_categories(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let result = Category::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_all_categories_with_default_pagination", None, None)]
#[case("can_find_all_categories_with_first_page", Some(2), Some(1))]
#[case("can_find_all_categories_with_second_page", Some(2), Some(2))]
#[case("can_find_all_categories_and_clamp_large_limit", Some(100), Some(1))]
#[case("can_find_all_categories_and_clamp_low_values", Some(0), Some(0))]
#[tokio::test]
#[serial]
async fn can_find_all_categories(
    #[case] test_name: &str,
    #[case] limit: Option<i64>,
    #[case] page: Option<i64>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Category::seed_data(ctx.db(), "categories.json")
        .await
        .expect("Failed to seed categories");

    let query = pagination_query(limit, page);
    let result = Category::find_all(ctx.db(), &query).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_all_categories_when_empty")]
#[tokio::test]
#[serial]
async fn can_find_all_categories_when_empty(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();
    let query = pagination_query(Some(10), Some(1));

    let result = Category::find_all(ctx.db(), &query).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_update_category",
    "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46",
    None,
    None,
    Some("Casual and formal trousers".to_string())
)]
#[case(
    "can_update_category_name_only",
    "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46",
    Some("Pants".to_string()),
    None,
    None
)]
#[case(
    "can_update_category_with_name_image_and_description",
    "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46",
    Some("Chinos".to_string()),
    Some("https://cdn.example.com/categories/chinos.png".to_string()),
    Some("Smart casual trousers".to_string())
)]
#[case(
    "can_update_category_and_normalize_name",
    "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46",
    Some("  Denim Jeans  ".to_string()),
    None,
    Some("Denim trousers".to_string())
)]
#[case(
    "can_update_category_with_same_name",
    "f63b79c9-4753-40c3-bc78-8c4fd38abd5b",
    Some("  T-SHIRTS  ".to_string()),
    Some("https://cdn.example.com/categories/tees.png".to_string()),
    Some("Short sleeve tops".to_string())
)]
#[case(
    "cannot_update_category_when_name_already_exists",
    "00b92bcb-cc7a-4a2b-bd80-e9c1b40d1c46",
    Some("T-shirts".to_string()),
    None,
    Some("Duplicate category".to_string())
)]
#[case(
    "cannot_update_category_when_category_does_not_exist",
    "00000000-0000-0000-0000-000000000000",
    Some("Outerwear".to_string()),
    None,
    Some("Jackets and coats".to_string())
)]
#[tokio::test]
#[serial]
async fn can_update_category(
    #[case] test_name: &str,
    #[case] pid: &str,
    #[case] name: Option<String>,
    #[case] image_link: Option<String>,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Category::seed_data(ctx.db(), "categories.json")
        .await
        .expect("Failed to seed categories");

    let params = update_category(name, image_link, None, description);
    let result = Category::update(ctx.db(), uuid(pid), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case(
    "can_find_tshirts_category_by_pid",
    "f63b79c9-4753-40c3-bc78-8c4fd38abd5b"
)]
#[case(
    "can_find_shoes_category_by_pid",
    "6f042674-322f-4933-afc8-1d3fd75599f6"
)]
#[tokio::test]
#[serial]
async fn can_find_category_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Category::seed_data(ctx.db(), "categories.json")
        .await
        .expect("Failed to seed categories");

    let result = Category::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "cannot_find_category_when_pid_does_not_exist",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn cannot_find_category_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Category::seed_data(ctx.db(), "categories.json")
        .await
        .expect("Failed to seed categories");

    let result = Category::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_category_by_normalized_name", " T-SHIRTS ".to_string())]
#[case("cannot_find_category_when_name_does_not_exist", "Outerwear".to_string())]
#[tokio::test]
#[serial]
async fn can_find_category_by_name(#[case] test_name: &str, #[case] name: String) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Category::seed_data(ctx.db(), "categories.json")
        .await
        .expect("Failed to seed categories");

    let result = Category::find_by_name(ctx.db(), &name).await;

    assert_debug_snapshot!(test_name, result);
}
