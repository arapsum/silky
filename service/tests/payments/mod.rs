use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use rust_decimal::Decimal;
use serial_test::serial;
use service::{
    config::StripeConfig,
    models::{CheckoutSessionDetails, Order, PaymentAttempt},
    payments::{process_stripe_event, to_minor_units, verify_stripe_event},
};
use stripe::{
    CheckoutSession, CheckoutSessionMode, CheckoutSessionPaymentStatus, Currency, Event,
    EventObject, EventType, NotificationEventData,
};

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

fn checkout_event(
    event_id: &str,
    event_type: EventType,
    session_id: &str,
    attempt_pid: uuid::Uuid,
    order_pid: uuid::Uuid,
    amount: i64,
    payment_status: CheckoutSessionPaymentStatus,
) -> Event {
    let session = CheckoutSession {
        id: session_id.parse().expect("session ID should parse"),
        amount_total: Some(amount),
        client_reference_id: Some(order_pid.to_string()),
        currency: Some(Currency::USD),
        expires_at: chrono::Utc::now().timestamp() + 1800,
        livemode: false,
        metadata: Some(std::collections::HashMap::from([
            ("order_pid".to_string(), order_pid.to_string()),
            ("payment_attempt_pid".to_string(), attempt_pid.to_string()),
        ])),
        mode: CheckoutSessionMode::Payment,
        payment_status,
        ..Default::default()
    };
    Event {
        id: event_id.parse().expect("event ID should parse"),
        api_version: Some("2025-02-24.acacia".to_string()),
        data: NotificationEventData {
            object: EventObject::CheckoutSession(session),
            previous_attributes: None,
        },
        livemode: false,
        type_: event_type,
        ..Default::default()
    }
}

#[rstest]
#[case(
    "completes_paid_checkout",
    EventType::CheckoutSessionCompleted,
    CheckoutSessionPaymentStatus::Paid,
    "succeeded",
    "confirmed",
    "paid"
)]
#[case(
    "waits_for_delayed_payment",
    EventType::CheckoutSessionCompleted,
    CheckoutSessionPaymentStatus::Unpaid,
    "processing",
    "pending",
    "pending"
)]
#[case(
    "completes_delayed_payment",
    EventType::CheckoutSessionAsyncPaymentSucceeded,
    CheckoutSessionPaymentStatus::Paid,
    "succeeded",
    "confirmed",
    "paid"
)]
#[case(
    "records_delayed_payment_failure",
    EventType::CheckoutSessionAsyncPaymentFailed,
    CheckoutSessionPaymentStatus::Unpaid,
    "failed",
    "pending",
    "failed"
)]
#[case(
    "expires_unpaid_checkout",
    EventType::CheckoutSessionExpired,
    CheckoutSessionPaymentStatus::Unpaid,
    "expired",
    "cancelled",
    "failed"
)]
#[case(
    "acknowledges_unknown_event",
    EventType::Unknown,
    CheckoutSessionPaymentStatus::Unpaid,
    "session_created",
    "pending",
    "pending"
)]
#[tokio::test]
#[serial]
async fn processes_checkout_webhooks_once(
    #[case] test_name: &str,
    #[case] event_type: EventType,
    #[case] payment_status: CheckoutSessionPaymentStatus,
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

    let event = checkout_event(
        &format!("evt_{test_name}"),
        event_type,
        &session_id,
        attempt.pid(),
        order_pid,
        to_minor_units(order.grand_total(), order.currency()).expect("amount should convert"),
        payment_status,
    );
    let expires_inventory = matches!(event_type, EventType::CheckoutSessionExpired);
    let payload = serde_json::to_value(&event).expect("event should serialize");
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
    if expires_inventory {
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
