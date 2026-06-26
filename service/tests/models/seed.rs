use insta::{Settings, assert_debug_snapshot};
use serial_test::serial;
use service::App;

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("seed");
        settings.set_snapshot_path("snapshots/seed");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_seed_data() {
    configure_insta!();

    let ctx = crate::boot_test().await.expect("Failed to initialise test");

    let result = App::seed(ctx.db()).await;

    assert_debug_snapshot!(result);
}
