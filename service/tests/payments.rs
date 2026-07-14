use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use rust_decimal::Decimal;
use service::payments::to_minor_units;

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/payments");
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
