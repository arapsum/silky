use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use service::{App, models::user::User, schemas::RegisterUser};
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_password, cleanup_uuid, cleanup_verification_token},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("users");
        settings.set_snapshot_path("snapshots/users");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_create_user() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let params = RegisterUser::new(
        Cow::Owned("test@mail.com".to_string()),
        Cow::Owned("test".to_string()),
        Cow::Owned("password".to_string()),
        Cow::Owned("password".to_string()),
    );

    let result = User::create(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_password());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[tokio::test]
#[serial]
async fn cannot_create_user_when_email_already_exists() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let params = RegisterUser::new(
        Cow::Owned("john.doe@acme.com".to_string()),
        Cow::Owned("John Doe".to_string()),
        Cow::Owned("password".to_string()),
        Cow::Owned("password".to_string()),
    );

    let result = User::create(ctx.db(), &params).await;

    assert_debug_snapshot!(result);
}

#[tokio::test]
#[serial]
async fn can_set_verification_token() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let mut user = User::find_by_claims_key(ctx.db(), "4c008e68-88fa-4072-808e-6888fa60724c")
        .await
        .unwrap();

    let token = Uuid::new_v4().to_string();

    let result = user
        .set_verification_token(
            ctx.db(),
            &token,
            ctx.config().auth().verification_token_expiry(),
        )
        .await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_verification_token());
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[tokio::test]
#[serial]
async fn can_find_user_by_email() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let result = User::find_by_email(ctx.db(), "john.doe@acme.com").await;

    with_settings!({
        filters => {
            let  filters = cleanup_verification_token().to_vec();
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[tokio::test]
#[serial]
async fn can_find_user_by_verification_token() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let result =
        User::find_by_verification_token(ctx.db(), "dbcbdf81-0230-4624-af68-e36c49d3f4ee").await;

    assert_debug_snapshot!(result)
}

#[tokio::test]
#[serial]
async fn can_verify_user() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let mut user = User::find_by_claims_key(ctx.db(), "4c008e68-88fa-4072-808e-6888fa60724c")
        .await
        .unwrap();

    let token = Uuid::new_v4().to_string();

    user.set_verification_token(
        ctx.db(),
        &token,
        ctx.config().auth().verification_token_expiry(),
    )
    .await
    .unwrap();

    let result = User::verify_email(ctx.db(), &token).await;

    with_settings!({
        filters => {
            let filters = cleanup_date().to_vec();
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}
