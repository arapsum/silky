use axum::http::{HeaderValue, StatusCode};
use axum_test::{TestRequest, TestServer};
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::{seed_data, utils};

const JANE_ADDRESS_PID: &str = "8d7d091d-51f8-4c86-86da-b1b0f9e87303";

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/addresses");
        settings.set_snapshot_suffix("addresses");
        let _guard = settings.bind_to_scope();
    };
}

fn response_filters() -> Vec<(&'static str, &'static str)> {
    let mut filters = utils::cleanup_date().to_vec();
    filters.extend(utils::cleanup_uuid().to_vec());
    filters.extend(utils::cleanup_headers());
    filters.push(("DATEZ", "DATE"));
    filters
}

async fn access_token(server: &TestServer, email: &str) -> HeaderValue {
    utils::login_users(
        server,
        &serde_json::json!({ "email": email, "password": "Password" }),
    )
    .await
    .access_token
}

fn with_auth(request: TestRequest, token: HeaderValue) -> TestRequest {
    let (name, value) = utils::auth_header(token);
    request.add_header(name, value)
}

fn address_payload() -> serde_json::Value {
    serde_json::json!({
        "addressType": "shipping",
        "label": "Weekend home",
        "recipientName": "Jane Smith",
        "lineOne": "42 Ocean View Road",
        "city": "Mombasa",
        "region": "Mombasa County",
        "postalCode": "80100",
        "countryCode": "ke",
        "email": "jane.smith@globex.com",
        "phone": "+254711000022",
        "isDefault": true
    })
}

#[derive(Clone, Copy)]
enum AddressActor {
    Anonymous,
    Staff,
}

#[rstest]
#[case("addresses_require_authentication", AddressActor::Anonymous)]
#[case("addresses_reject_staff_accounts", AddressActor::Staff)]
#[tokio::test]
#[serial]
async fn protects_customer_addresses(#[case] test_name: &str, #[case] actor: AddressActor) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");

        let mut request = server.get("/addresses");
        if matches!(actor, AddressActor::Staff) {
            request = with_auth(request, access_token(&server, "admin@silk.com").await);
        }

        let response = request.await;
        assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
    })
    .await;
}

#[rstest]
#[case("customer_can_list_own_addresses", "/addresses", StatusCode::OK)]
#[case(
    "customer_can_fetch_own_address",
    "/addresses/8d7d091d-51f8-4c86-86da-b1b0f9e87303",
    StatusCode::OK
)]
#[case(
    "customer_cannot_fetch_another_customers_address",
    "/addresses/4f3d4f3e-1c26-4f5f-a54f-6b5b2b8a7301",
    StatusCode::NOT_FOUND
)]
#[tokio::test]
#[serial]
async fn scopes_addresses_to_the_customer(
    #[case] test_name: &str,
    #[case] path: &str,
    #[case] expected_status: StatusCode,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");

        let response = with_auth(
            server.get(path),
            access_token(&server, "jane.smith@globex.com").await,
        )
        .await;

        assert_eq!(response.status_code(), expected_status);
        with_settings!({ filters => response_filters() }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn customer_can_create_update_and_delete_an_address() {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let token = access_token(&server, "jane.smith@globex.com").await;

        let created = with_auth(
            server.post("/addresses").json(&address_payload()),
            token.clone(),
        )
        .await;
        assert_eq!(created.status_code(), StatusCode::CREATED);
        let created_body: serde_json::Value = created.json();
        let pid = created_body["pid"]
            .as_str()
            .expect("created address should have a pid");

        let previous_default = with_auth(
            server.get(&format!("/addresses/{JANE_ADDRESS_PID}")),
            token.clone(),
        )
        .await
        .json::<serde_json::Value>();
        assert_eq!(previous_default["isDefault"], false);

        let mut replacement = address_payload();
        replacement["label"] = serde_json::json!("Primary home");
        replacement["lineTwo"] = serde_json::json!("Apartment 7");
        let updated = with_auth(
            server.put(&format!("/addresses/{pid}")).json(&replacement),
            token.clone(),
        )
        .await;
        assert_eq!(updated.status_code(), StatusCode::OK);

        let deleted = with_auth(server.delete(&format!("/addresses/{pid}")), token.clone()).await;
        assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);

        let missing = with_auth(server.get(&format!("/addresses/{pid}")), token).await;
        assert_eq!(missing.status_code(), StatusCode::NOT_FOUND);

        with_settings!({ filters => response_filters() }, {
            assert_debug_snapshot!(
                "customer_can_create_update_and_delete_an_address",
                (
                    created.status_code(),
                    created_body,
                    updated.status_code(),
                    updated.text(),
                    deleted.status_code(),
                    missing.status_code(),
                    missing.text(),
                )
            );
        });
    })
    .await;
}

#[rstest]
#[case(
    "rejects_invalid_address_type",
    "addressType",
    serde_json::json!("collection"),
    StatusCode::BAD_REQUEST
)]
#[case(
    "rejects_invalid_address_country",
    "countryCode",
    serde_json::json!("KEN"),
    StatusCode::BAD_REQUEST
)]
#[case(
    "rejects_customer_identity_in_address_payload",
    "customerPid",
    serde_json::json!("bd6f7c26-d2c9-487e-b837-8f77be468033"),
    StatusCode::UNPROCESSABLE_ENTITY
)]
#[tokio::test]
#[serial]
async fn rejects_invalid_address_payloads(
    #[case] test_name: &str,
    #[case] field: &str,
    #[case] value: serde_json::Value,
    #[case] expected_status: StatusCode,
) {
    crate::request(|server, ctx| async move {
        configure_insta!();
        seed_data(ctx.db()).await.expect("seed should complete");
        let mut payload = address_payload();
        payload[field] = value;

        let response = with_auth(
            server.post("/addresses").json(&payload),
            access_token(&server, "jane.smith@globex.com").await,
        )
        .await;

        assert_eq!(response.status_code(), expected_status);
        assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
    })
    .await;
}
