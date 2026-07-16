use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use rust_decimal::Decimal;
use serial_test::serial;
use service::{
    config::StripeConfig,
    models::{CheckoutSessionDetails, Order, PaymentAttempt},
    payments::{process_stripe_event, to_minor_units, verify_stripe_event},
};
use stripe_webhook::{Event, EventObject, Webhook};

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("../snapshots/payments");
        settings.set_snapshot_suffix("payments");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case("converts_zero", Decimal::ZERO, "USD")]
#[case("converts_whole_dollars", Decimal::new(100, 2), "USD")]
#[case("converts_cents", Decimal::new(2499, 2), "usd")]
#[case("rejects_fractional_cents", Decimal::new(24991, 3), "USD")]
#[case("rejects_negative_amounts", Decimal::NEGATIVE_ONE, "USD")]
#[case("rejects_unsupported_currency", Decimal::ONE, "KES")]
fn converts_checkout_amounts(
    #[case] test_name: &str,
    #[case] amount: Decimal,
    #[case] currency: &str,
) {
    configure_insta!();
    assert_debug_snapshot!(test_name, to_minor_units(amount, currency));
}

#[test]
fn rejects_invalid_webhook_signatures() {
    configure_insta!();
    let result = verify_stripe_event("{}", "t=1720950000,v1=invalid", "whsec_test");
    assert_debug_snapshot!("rejects_invalid_webhook_signatures", result);
}

fn stripe_config() -> StripeConfig {
    serde_json::from_value(serde_json::json!({
        "secret_key": "sk_test_placeholder",
        "webhook_secret": "whsec_placeholder",
        "checkout_success_url": "http://localhost:3000/checkout/success",
        "checkout_cancel_url": "http://localhost:3000/checkout/cancel",
        "checkout_ttl_seconds": 1800,
        "live_mode": false
    }))
    .expect("test Stripe config should deserialize")
}

fn checkout_event_payload(
    event_id: &str,
    event_type: &str,
    session_id: &str,
    attempt_pid: uuid::Uuid,
    order_pid: uuid::Uuid,
    amount: i64,
    payment_status: &str,
) -> String {
    serde_json::json!({
        "id": event_id,
        "object": "event",
        "api_version": "2026-06-24.dahlia",
        "created": chrono::Utc::now().timestamp(),
        "data": {
            "object": {
                "id": session_id,
                "object": "checkout.session",
                "amount_total": amount,
                "automatic_tax": { "enabled": false },
                "client_reference_id": order_pid,
                "created": chrono::Utc::now().timestamp(),
                "currency": "usd",
                "custom_fields": [],
                "custom_text": {},
                "expires_at": chrono::Utc::now().timestamp() + 1800,
                "livemode": false,
                "metadata": {
                    "order_pid": order_pid,
                    "payment_attempt_pid": attempt_pid,
                },
                "mode": "payment",
                "payment_intent": "pi_test_silk",
                "payment_method_types": ["card"],
                "payment_status": payment_status,
                "shipping_options": [],
                "status": "complete",
                "ui_mode": "hosted_page"
            }
        },
        "livemode": false,
        "pending_webhooks": 1,
        "request": { "id": null, "idempotency_key": null },
        "type": event_type
    })
    .to_string()
}

fn verified_checkout_event(payload: &str) -> Event {
    let secret = "whsec_test_silk";
    let signature = Webhook::generate_test_header(payload, secret, None);
    verify_stripe_event(payload, &signature, secret)
        .expect("current Stripe webhook fixture should verify and deserialize")
}

#[test]
fn accepts_the_current_stripe_checkout_snapshot_shape() {
    let attempt_pid = uuid::Uuid::new_v4();
    let order_pid = uuid::Uuid::new_v4();
    let payload = checkout_event_payload(
        "evt_current_checkout_snapshot",
        "checkout.session.completed",
        "cs_test_current_checkout_snapshot",
        attempt_pid,
        order_pid,
        2_499,
        "paid",
    );
    let event = verified_checkout_event(&payload);

    assert_eq!(
        event.api_version.as_ref().map(ToString::to_string),
        Some("2026-06-24.dahlia".to_string())
    );
    let EventObject::CheckoutSessionCompleted(session) = event.data.object else {
        panic!("fixture should deserialize as a completed Checkout Session");
    };
    assert_eq!(
        session.client_reference_id.as_deref(),
        Some(order_pid.to_string()).as_deref()
    );
    assert_eq!(
        session.ui_mode.map(|mode| mode.to_string()).as_deref(),
        Some("hosted_page")
    );
}

#[test]
fn classifies_a_signed_malformed_webhook_as_an_invalid_payload() {
    let payload = r#"{"object":"event"}"#;
    let secret = "whsec_test_silk";
    let signature = Webhook::generate_test_header(payload, secret, None);
    let error = verify_stripe_event(payload, &signature, secret)
        .expect_err("an incomplete signed event must not deserialize");

    assert_eq!(error.code(), "invalid_stripe_payload");
    assert!(error.to_string().contains("webhook payload is invalid"));
}

#[rstest]
#[case(
    "completes_paid_checkout",
    "checkout.session.completed",
    "paid",
    "succeeded",
    "confirmed",
    "paid"
)]
#[case(
    "waits_for_delayed_payment",
    "checkout.session.completed",
    "unpaid",
    "processing",
    "pending",
    "pending"
)]
#[case(
    "completes_delayed_payment",
    "checkout.session.async_payment_succeeded",
    "paid",
    "succeeded",
    "confirmed",
    "paid"
)]
#[case(
    "records_delayed_payment_failure",
    "checkout.session.async_payment_failed",
    "unpaid",
    "failed",
    "cancelled",
    "failed"
)]
#[case(
    "expires_unpaid_checkout",
    "checkout.session.expired",
    "unpaid",
    "expired",
    "cancelled",
    "failed"
)]
#[case(
    "acknowledges_unknown_event",
    "silk.test.unknown",
    "unpaid",
    "session_created",
    "pending",
    "pending"
)]
#[tokio::test]
#[serial]
async fn processes_checkout_webhooks_once(
    #[case] test_name: &str,
    #[case] event_type: &str,
    #[case] payment_status: &str,
    #[case] expected_attempt_status: &str,
    #[case] expected_order_status: &str,
    #[case] expected_payment_status: &str,
) {
    configure_insta!();
    let ctx = crate::boot_test().await.expect("test context should boot");
    crate::seed_data(ctx.db())
        .await
        .expect("seed should complete");
    let order_pid = uuid::Uuid::parse_str("26a12ef5-1bd0-4673-b772-96f4f4f94007")
        .expect("order PID should parse");
    let order = Order::find_by_pid(ctx.db(), order_pid)
        .await
        .expect("order should exist");
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    let attempt = PaymentAttempt::create_for_order(
        &mut txn,
        order.row_id(),
        order.grand_total(),
        order.currency(),
    )
    .await
    .expect("attempt should be created");
    let session_id = format!("cs_test_{test_name}");
    PaymentAttempt::attach_checkout_session(
        &mut txn,
        attempt.pid(),
        &CheckoutSessionDetails {
            session_id: &session_id,
            checkout_url: "https://checkout.stripe.test/session",
            expires_at: chrono::Utc::now().fixed_offset(),
        },
    )
    .await
    .expect("session should attach");
    txn.commit().await.expect("transaction should commit");

    let payload = checkout_event_payload(
        &format!("evt_{test_name}"),
        event_type,
        &session_id,
        attempt.pid(),
        order_pid,
        to_minor_units(order.grand_total(), order.currency()).expect("amount should convert"),
        payment_status,
    );
    let event = verified_checkout_event(&payload);
    let releases_inventory = matches!(
        event_type,
        "checkout.session.expired" | "checkout.session.async_payment_failed"
    );
    let payload = serde_json::from_str(&payload).expect("event payload should be valid JSON");
    let stock_before = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(stock_quantity), 0)::BIGINT FROM product_variants",
    )
    .fetch_one(ctx.db())
    .await
    .expect("stock total should load");
    process_stripe_event(ctx.db(), &stripe_config(), event.clone(), &payload)
        .await
        .expect("first delivery should process");
    let stock_after_first = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(stock_quantity), 0)::BIGINT FROM product_variants",
    )
    .fetch_one(ctx.db())
    .await
    .expect("stock total should load");
    process_stripe_event(ctx.db(), &stripe_config(), event, &payload)
        .await
        .expect("duplicate delivery should be acknowledged");
    let stock_after_duplicate = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(stock_quantity), 0)::BIGINT FROM product_variants",
    )
    .fetch_one(ctx.db())
    .await
    .expect("stock total should load");
    assert_eq!(stock_after_first, stock_after_duplicate);
    if releases_inventory {
        assert!(stock_after_first > stock_before);
    } else {
        assert_eq!(stock_after_first, stock_before);
    }

    let stored_attempt = PaymentAttempt::find_by_pid(ctx.db(), attempt.pid())
        .await
        .expect("attempt should exist");
    let lifecycle = sqlx::query_as::<_, (String, String)>(
        "SELECT status, payment_status FROM orders WHERE pid = $1",
    )
    .bind(order_pid)
    .fetch_one(ctx.db())
    .await
    .expect("order lifecycle should load");
    assert_eq!(stored_attempt.status(), expected_attempt_status);
    assert_eq!(lifecycle.0, expected_order_status);
    assert_eq!(lifecycle.1, expected_payment_status);
    assert_debug_snapshot!(
        test_name,
        (stored_attempt.status(), lifecycle.0, lifecycle.1)
    );
}
