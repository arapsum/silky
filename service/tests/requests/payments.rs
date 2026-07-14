use insta::{Settings, assert_debug_snapshot};

macro_rules! configure_insta {
    () => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/payments");
        settings.set_snapshot_suffix("payment_requests");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
async fn stripe_webhook_requires_a_signature() {
    configure_insta!();
    crate::request(|server, _ctx| async move {
        let response = server.post("/payments/stripe/webhook").text("{}").await;
        assert_debug_snapshot!(
            "stripe_webhook_requires_a_signature",
            (response.status_code(), response.text())
        );
    })
    .await;
}
