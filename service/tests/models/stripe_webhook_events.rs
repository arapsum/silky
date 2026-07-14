use insta::{Settings, assert_debug_snapshot};
use serial_test::serial;
use service::models::StripeWebhookEvent;

use crate::boot_test;

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("stripe_webhook_events");
        settings.set_snapshot_path("snapshots/stripe_webhook_events");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn records_each_stripe_event_once() {
    configure_insta!();
    let ctx = boot_test().await.expect("test context should boot");
    let payload = serde_json::json!({
        "id": "evt_test_completed",
        "type": "checkout.session.completed",
        "data": { "object": { "id": "cs_test_completed" } }
    });
    let mut txn = ctx.db().begin().await.expect("transaction should begin");
    let first = StripeWebhookEvent::record_once(
        &mut txn,
        "evt_test_completed",
        "checkout.session.completed",
        Some("2025-02-24.acacia"),
        &payload,
    )
    .await
    .expect("first event should record");
    let duplicate = StripeWebhookEvent::record_once(
        &mut txn,
        "evt_test_completed",
        "checkout.session.completed",
        Some("2025-02-24.acacia"),
        &payload,
    )
    .await
    .expect("duplicate should be acknowledged");
    txn.commit().await.expect("transaction should commit");
    let stored = StripeWebhookEvent::find_by_id(ctx.db(), "evt_test_completed")
        .await
        .expect("event lookup should succeed");
    let mut stored = serde_json::to_value(stored).expect("stored event should serialize");
    let stored = stored
        .as_object_mut()
        .expect("stored event should be an object");
    stored.remove("processedAt");
    stored.remove("createdAt");

    assert_debug_snapshot!("records_each_stripe_event_once", (first, duplicate, stored));
}
