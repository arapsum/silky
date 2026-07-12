use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use service::schemas::{NewAddress, NewOrder, Validator};
use uuid::Uuid;

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("orders");
        settings.set_snapshot_path("snapshots/orders");
        let _guard = settings.bind_to_scope();
    };
}

fn address(value: serde_json::Value) -> NewAddress {
    serde_json::from_value(value).expect("address should deserialize")
}

#[rstest]
#[case("invalid address type", "collection", "address_type")]
#[case("invalid country code", "KE1", "country_code")]
#[case("invalid email", "not-an-email", "email")]
fn rejects_invalid_addresses(#[case] name: &str, #[case] value: &str, #[case] expected: &str) {
    configure_insta!();
    let mut payload = serde_json::json!({
        "customerPid": Uuid::nil(),
        "addressType": "shipping",
        "recipientName": "John Doe",
        "lineOne": "10 Market Street",
        "city": "Nairobi",
        "countryCode": "KE",
        "email": "john@example.com",
        "isDefault": false
    });
    match name {
        "invalid address type" => payload["addressType"] = serde_json::json!(value),
        "invalid country code" => payload["countryCode"] = serde_json::json!(value),
        "invalid email" => payload["email"] = serde_json::json!(value),
        _ => unreachable!(),
    }
    let input = address(payload);
    let validator = Validator::new(input);
    let result = validator.validate();
    let error = result.expect_err("invalid address should fail").to_string();
    assert!(error.contains(expected), "{error}");
    assert_debug_snapshot!(name, error);
}

#[rstest]
#[case("empty order", serde_json::json!([]))]
#[case(
    "zero item quantity",
    serde_json::json!([{
        "variantPid": Uuid::nil(),
        "quantity": 0
    }])
)]
fn rejects_invalid_orders(#[case] name: &str, #[case] items: serde_json::Value) {
    configure_insta!();
    let payload = serde_json::json!({
        "customerPid": Uuid::nil(),
        "items": items
    });
    let input = serde_json::from_value::<NewOrder>(payload).expect("order should deserialize");
    let validator = Validator::new(input);
    let result = validator.validate();
    let result = result
        .map(|_| "valid".to_string())
        .map_err(|error| error.to_string());
    assert_debug_snapshot!(name, result);
}
