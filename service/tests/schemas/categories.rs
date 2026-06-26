use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use service::schemas::{NewCategory, UpdateCategory, Validator};

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

#[rstest]
#[case(
    "new_category_validation_accepts_valid_params",
    "T-shirts".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_accepts_missing_description",
    "T-shirts".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    None
)]
#[case(
    "new_category_validation_accepts_parent_id",
    "Graphic Tees".to_string(),
    "https://cdn.example.com/categories/graphic-tees.png".to_string(),
    Some(101),
    None
)]
#[case(
    "new_category_validation_rejects_empty_name",
    "".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_rejects_short_name",
    "A".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_rejects_long_name",
    "a".repeat(49),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_rejects_name_with_special_chars",
    "T-shirts+".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_rejects_invalid_image_link",
    "T-shirts".to_string(),
    "not-a-url".to_string(),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "new_category_validation_rejects_invalid_parent_id",
    "Graphic Tees".to_string(),
    "https://cdn.example.com/categories/graphic-tees.png".to_string(),
    Some(0),
    None
)]
#[case(
    "new_category_validation_rejects_long_description",
    "T-shirts".to_string(),
    "https://cdn.example.com/categories/tshirts.png".to_string(),
    None,
    Some("a".repeat(1001))
)]
fn can_validate_new_category(
    #[case] test_name: &str,
    #[case] name: String,
    #[case] image_link: String,
    #[case] parent_id: Option<i32>,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let params = new_category(name, image_link, parent_id, description);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "update_category_validation_accepts_valid_name",
    Some("T-shirts".to_string()),
    None,
    None,
    None
)]
#[case(
    "update_category_validation_accepts_image_link_only",
    None,
    Some("https://cdn.example.com/categories/tshirts.png".to_string()),
    None,
    None
)]
#[case(
    "update_category_validation_accepts_parent_id",
    None,
    None,
    Some(101),
    None
)]
#[case(
    "update_category_validation_accepts_description_only",
    None,
    None,
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_empty_name",
    Some("".to_string()),
    None,
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_short_name",
    Some("A".to_string()),
    None,
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_long_name",
    Some("a".repeat(49)),
    None,
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_name_with_special_chars",
    Some("T-shirts+".to_string()),
    None,
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_invalid_image_link",
    None,
    Some("not-a-url".to_string()),
    None,
    Some("Short sleeve tops".to_string())
)]
#[case(
    "update_category_validation_rejects_invalid_parent_id",
    None,
    None,
    Some(0),
    None
)]
#[case(
    "update_category_validation_rejects_long_description",
    Some("T-shirts".to_string()),
    None,
    None,
    Some("a".repeat(1001))
)]
fn can_validate_update_category(
    #[case] test_name: &str,
    #[case] name: Option<String>,
    #[case] image_link: Option<String>,
    #[case] parent_id: Option<i32>,
    #[case] description: Option<String>,
) {
    configure_insta!();

    let params = update_category(name, image_link, parent_id, description);
    let result = Validator::new(params)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}
