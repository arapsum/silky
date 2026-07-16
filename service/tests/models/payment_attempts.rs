use chrono::DateTime;
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use rust_decimal::Decimal;
use serial_test::serial;
use service::{
    models::{CheckoutSessionDetails, ModelError, Order, PaymentAttempt},
    schemas::NewOrder,
};
use uuid::Uuid;

use crate::{
    boot_test, seed_data,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("payment_attempts");
        settings.set_snapshot_path("snapshots/payment_attempts");
        let _guard = settings.bind_to_scope();
    };
}

fn checkout_order() -> NewOrder {
    serde_json::from_value(serde_json::json!({
        "customerPid": "bd6f7c26-d2c9-487e-b837-8f77be468033",
        "items": [{
            "variantPid": "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
            "quantity": 2
        }]
    }))
    .expect("checkout order should deserialize")
}

fn session<'a>(session_id: &'a str) -> CheckoutSessionDetails<'a> {
    CheckoutSessionDetails {
        session_id,
        checkout_url: "https://checkout.stripe.com/c/pay/test-session",
        expires_at: DateTime::parse_from_rfc3339("2026-07-14T12:30:00+03:00")
            .expect("expiry should parse"),
    }
}

async fn create_order_and_attempt(ctx: &service::AppContext) -> (Order, PaymentAttempt, i32) {
    let order = Order::create(ctx.db(), &checkout_order())
        .await
        .expect("order should create");
    let order_id = sqlx::query_scalar::<_, i32>("SELECT id FROM orders WHERE pid = $1")
        .bind(order.pid())
        .fetch_one(ctx.db())
        .await
        .expect("order id should load");
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    let attempt =
        PaymentAttempt::create_for_order(&mut txn, order_id, Decimal::new(17_998, 2), "USD")
            .await
            .expect("payment attempt should create");
    txn.commit().await.expect("transaction should commit");
    (order, attempt, order_id)
}

#[tokio::test]
#[serial]
async fn creates_one_active_attempt_per_order() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let (_order, attempt, order_id) = create_order_and_attempt(&ctx).await;

    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    let duplicate =
        PaymentAttempt::create_for_order(&mut txn, order_id, Decimal::new(17_998, 2), "USD")
            .await
            .map_err(|error| (error.code(), error.to_string()));
    txn.rollback().await.expect("transaction should roll back");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("creates_one_active_attempt_per_order", (attempt, duplicate));
    });
}

#[tokio::test]
#[serial]
async fn attaching_the_same_checkout_session_is_idempotent() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let (_order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    let attached = PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_idempotent"),
    )
    .await
    .expect("session should attach");
    let replayed = PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_idempotent"),
    )
    .await
    .expect("same session should be idempotent");
    txn.commit().await.expect("transaction should commit");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("attaching_the_same_checkout_session_is_idempotent", (attached, replayed));
    });
}

#[rstest]
#[case(
    "customer_can_find_owned_checkout_session",
    "bd6f7c26-d2c9-487e-b837-8f77be468033"
)]
#[case(
    "customer_cannot_find_another_customers_checkout_session",
    "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8"
)]
#[tokio::test]
#[serial]
async fn scopes_checkout_session_lookup_to_the_customer(
    #[case] test_name: &str,
    #[case] customer_pid: &str,
) {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let (order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &CheckoutSessionDetails {
            session_id: "cs_test_customer_lookup",
            checkout_url: "https://checkout.stripe.com/c/pay/customer-lookup",
            expires_at: DateTime::parse_from_rfc3339("2027-07-15T12:00:00+03:00")
                .expect("expiry should parse"),
        },
    )
    .await
    .expect("session should attach");
    txn.commit().await.expect("transaction should commit");

    let customer_pid = Uuid::parse_str(customer_pid).expect("customer pid should parse");
    let result =
        PaymentAttempt::find_checkout_session_for_customer(ctx.db(), order.pid(), customer_pid)
            .await;

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!(test_name, result);
    });
}

#[tokio::test]
#[serial]
async fn expired_checkout_session_does_not_expose_redirect_url() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let (order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &CheckoutSessionDetails {
            session_id: "cs_test_expired_lookup",
            checkout_url: "https://checkout.stripe.com/c/pay/expired",
            expires_at: DateTime::parse_from_rfc3339("2025-07-15T12:00:00+03:00")
                .expect("expiry should parse"),
        },
    )
    .await
    .expect("session should attach");
    txn.commit().await.expect("transaction should commit");

    let customer_pid =
        Uuid::parse_str("bd6f7c26-d2c9-487e-b837-8f77be468033").expect("customer pid should parse");
    let result =
        PaymentAttempt::find_checkout_session_for_customer(ctx.db(), order.pid(), customer_pid)
            .await;

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("expired_checkout_session_does_not_expose_redirect_url", result);
    });
}

#[derive(Clone, Copy)]
enum ForbiddenTransition {
    SucceededToProcessing,
    FailedToSucceeded,
}

#[rstest]
#[case("succeeded to processing", ForbiddenTransition::SucceededToProcessing)]
#[case("failed to succeeded", ForbiddenTransition::FailedToSucceeded)]
#[tokio::test]
#[serial]
async fn rejects_terminal_payment_state_regressions(
    #[case] snapshot: &str,
    #[case] transition: ForbiddenTransition,
) {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let (_order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_transition"),
    )
    .await
    .expect("session should attach");

    let result = match transition {
        ForbiddenTransition::SucceededToProcessing => {
            PaymentAttempt::mark_succeeded(&mut txn, "cs_test_transition", Some("pi_test"))
                .await
                .expect("success should apply");
            PaymentAttempt::mark_processing(&mut txn, "cs_test_transition").await
        }
        ForbiddenTransition::FailedToSucceeded => {
            PaymentAttempt::fail_and_release_inventory(
                &mut txn,
                "cs_test_transition",
                Some("card_declined"),
                Some("Payment was declined"),
            )
            .await
            .expect("failure should apply");
            PaymentAttempt::mark_succeeded(&mut txn, "cs_test_transition", Some("pi_test")).await
        }
    }
    .map_err(|error| (error.code(), error.to_string()));
    txn.rollback().await.expect("transaction should roll back");

    assert_debug_snapshot!(snapshot, result);
}

#[tokio::test]
#[serial]
async fn expiration_releases_reserved_inventory_exactly_once() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let variant_pid = Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("variant pid");
    let stock_before =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("stock should load");
    let (order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let stock_reserved =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("reserved stock should load");

    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(&mut txn, attempt.pid(), &session("cs_test_expiry"))
        .await
        .expect("session should attach");
    let expired = PaymentAttempt::expire_and_release_inventory(&mut txn, "cs_test_expiry")
        .await
        .expect("attempt should expire");
    let replayed = PaymentAttempt::expire_and_release_inventory(&mut txn, "cs_test_expiry")
        .await
        .expect("expiry replay should be idempotent");
    txn.commit().await.expect("transaction should commit");

    let stock_after =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("restored stock should load");
    let state = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, payment_status, fulfillment_status FROM orders WHERE pid = $1",
    )
    .bind(order.pid())
    .fetch_one(ctx.db())
    .await
    .expect("order state should load");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("expiration_releases_reserved_inventory_exactly_once", (
            expired,
            replayed,
            stock_before,
            stock_reserved,
            stock_after,
            state,
        ));
    });
}

#[tokio::test]
#[serial]
async fn asynchronous_failure_releases_reserved_inventory_exactly_once() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let variant_pid =
        Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("variant pid should parse");
    let stock_before =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("stock should load");
    let (order, attempt, _order_id) = create_order_and_attempt(&ctx).await;
    let stock_reserved =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("reserved stock should load");

    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_async_failure"),
    )
    .await
    .expect("session should attach");
    let failed = PaymentAttempt::fail_and_release_inventory(
        &mut txn,
        "cs_test_async_failure",
        Some("async_payment_failed"),
        None,
    )
    .await
    .expect("attempt should fail");
    let replayed = PaymentAttempt::fail_and_release_inventory(
        &mut txn,
        "cs_test_async_failure",
        Some("async_payment_failed"),
        None,
    )
    .await
    .expect("failure replay should be idempotent");
    txn.commit().await.expect("transaction should commit");

    let stock_after =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("restored stock should load");
    let state = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, payment_status, fulfillment_status FROM orders WHERE pid = $1",
    )
    .bind(order.pid())
    .fetch_one(ctx.db())
    .await
    .expect("order state should load");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("asynchronous_failure_releases_reserved_inventory_exactly_once", (
            failed,
            replayed,
            stock_before,
            stock_reserved,
            stock_after,
            state,
        ));
    });
}

#[tokio::test]
#[serial]
async fn late_failure_cannot_revert_a_successful_payment() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let variant_pid =
        Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("variant pid should parse");
    let (_order, attempt, order_id) = create_order_and_attempt(&ctx).await;
    let stock_reserved =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("reserved stock should load");

    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_paid_then_failed"),
    )
    .await
    .expect("session should attach");
    let succeeded =
        PaymentAttempt::mark_succeeded(&mut txn, "cs_test_paid_then_failed", Some("pi_test_paid"))
            .await
            .expect("payment should succeed");
    let after_failure = PaymentAttempt::fail_and_release_inventory(
        &mut txn,
        "cs_test_paid_then_failed",
        Some("late_failure"),
        None,
    )
    .await
    .expect("late failure should be acknowledged");
    txn.commit().await.expect("transaction should commit");

    let stock_after =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("stock should remain reserved");
    let state = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, payment_status, fulfillment_status FROM orders WHERE id = $1",
    )
    .bind(order_id)
    .fetch_one(ctx.db())
    .await
    .expect("order state should load");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("late_failure_cannot_revert_a_successful_payment", (
            succeeded,
            after_failure,
            stock_reserved,
            stock_after,
            state,
        ));
    });
}

#[tokio::test]
#[serial]
async fn late_expiry_cannot_revert_a_successful_payment() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    seed_data(ctx.db()).await.expect("seed should complete");
    let variant_pid = Uuid::parse_str("db365773-2ac1-49aa-a4b9-03dcf8ac3401").expect("variant pid");
    let (_order, attempt, order_id) = create_order_and_attempt(&ctx).await;
    let stock_reserved =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("reserved stock should load");

    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &session("cs_test_paid_then_expired"),
    )
    .await
    .expect("session should attach");
    let succeeded =
        PaymentAttempt::mark_succeeded(&mut txn, "cs_test_paid_then_expired", Some("pi_test_paid"))
            .await
            .expect("payment should succeed");
    let after_expiry =
        PaymentAttempt::expire_and_release_inventory(&mut txn, "cs_test_paid_then_expired")
            .await
            .expect("late expiry should be acknowledged");
    txn.commit().await.expect("transaction should commit");

    let stock_after =
        sqlx::query_scalar::<_, i32>("SELECT stock_quantity FROM product_variants WHERE pid = $1")
            .bind(variant_pid)
            .fetch_one(ctx.db())
            .await
            .expect("stock should remain reserved");
    let state = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, payment_status, fulfillment_status FROM orders WHERE id = $1",
    )
    .bind(order_id)
    .fetch_one(ctx.db())
    .await
    .expect("order state should load");

    with_settings!({ filters => { let mut filters = cleanup_uuid().to_vec(); filters.extend(cleanup_date()); filters.extend(cleanup_id()); filters } }, {
        assert_debug_snapshot!("late_expiry_cannot_revert_a_successful_payment", (
            succeeded,
            after_expiry,
            stock_reserved,
            stock_after,
            state,
        ));
    });
}

#[test]
fn payment_attempt_errors_expose_stable_metadata() {
    configure_insta!();
    let errors = [
        ModelError::PaymentInProgress,
        ModelError::PaymentAttemptNotFound,
        ModelError::InvalidPaymentTransition {
            current: "succeeded".to_string(),
            target: "expired",
        },
    ];
    let metadata = errors
        .iter()
        .map(|error| {
            let (status, message) = error.response_body();
            (error.code(), error.field(), status, message)
        })
        .collect::<Vec<_>>();

    assert_debug_snapshot!("payment_attempt_error_metadata", metadata);
}
