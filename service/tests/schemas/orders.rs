use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use service::schemas::{NewAddress, NewOrder, NewOrderDetail, Validator};
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
#[case("bad currency", serde_json::json!("US"))]
#[case("negative subtotal", serde_json::json!(-1))]
#[case("negative grand total", serde_json::json!(-1))]
fn rejects_invalid_orders(#[case] name: &str, #[case] value: serde_json::Value) {
    configure_insta!();
    let mut payload = serde_json::json!({
        "customerPid": Uuid::nil(),
        "currency": "USD",
        "subtotal": "10.00",
        "discountTotal": "0.00",
        "shippingTotal": "0.00",
        "taxTotal": "0.00",
        "grandTotal": "10.00"
    });
    match name {
        "bad currency" => payload["currency"] = value,
        "negative subtotal" => payload["subtotal"] = value,
        "negative grand total" => payload["grandTotal"] = value,
        _ => unreachable!(),
    }
    let input = serde_json::from_value::<NewOrder>(payload).expect("order should deserialize");
    let validator = Validator::new(input);
    let result = validator.validate();
    let result = result
        .map(|_| "valid".to_string())
        .map_err(|error| error.to_string());
    assert_debug_snapshot!(name, result);
}

#[test]
fn rejects_zero_order_detail_quantity() {
    configure_insta!();
    let input: NewOrderDetail = serde_json::from_value(serde_json::json!({
        "orderPid": Uuid::nil(),
        "variantPid": Uuid::nil(),
        "quantity": 0,
        "discountTotal": "0.00",
        "taxTotal": "0.00"
    }))
    .expect("detail should deserialize");
    let result = Validator::new(input)
        .validate()
        .map(|_| "valid".to_string())
        .map_err(|error| error.to_string());
    assert_debug_snapshot!("rejects_zero_order_detail_quantity", result);
}
