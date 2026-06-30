use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serde_json::{Value, json};
use service::schemas::{PaginationQuery, PermissionRoleQuery, Validator};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("request");
        settings.set_snapshot_path("snapshots/request");
        let _guard = settings.bind_to_scope();
    };
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

fn permission_role_query(role_id: Option<i32>, permission_id: Option<i32>) -> PermissionRoleQuery {
    let mut value = serde_json::Map::new();

    if let Some(role_id) = role_id {
        value.insert("role_id".to_string(), json!(role_id));
    }

    if let Some(permission_id) = permission_id {
        value.insert("permission_id".to_string(), json!(permission_id));
    }

    serde_json::from_value(Value::Object(value)).expect("Failed to parse role permission query")
}

#[rstest]
#[case("pagination_query_validation_accepts_missing_params", None, None)]
#[case("pagination_query_validation_accepts_valid_params", Some(20), Some(2))]
#[case("pagination_query_validation_rejects_zero_limit", Some(0), Some(1))]
#[case("pagination_query_validation_rejects_zero_page", Some(20), Some(0))]
#[case("pagination_query_validation_rejects_negative_params", Some(-1), Some(-2))]
fn can_validate_pagination_query(
    #[case] test_name: &str,
    #[case] limit: Option<i64>,
    #[case] page: Option<i64>,
) {
    configure_insta!();

    let query = pagination_query(limit, page);
    let result = Validator::new(query)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("permission_role_query_validation_accepts_missing_params", None, None)]
#[case(
    "permission_role_query_validation_accepts_valid_params",
    Some(11),
    Some(101)
)]
#[case(
    "permission_role_query_validation_rejects_zero_role_id",
    Some(0),
    Some(101)
)]
#[case(
    "permission_role_query_validation_rejects_zero_permission_id",
    Some(11),
    Some(0)
)]
#[case("permission_role_query_validation_rejects_negative_params", Some(-1), Some(-2))]
fn can_validate_permission_role_query(
    #[case] test_name: &str,
    #[case] role_id: Option<i32>,
    #[case] permission_id: Option<i32>,
) {
    configure_insta!();

    let query = permission_role_query(role_id, permission_id);
    let result = Validator::new(query)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|err| err.to_string());

    assert_debug_snapshot!(test_name, result);
}
