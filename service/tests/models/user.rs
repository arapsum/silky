use std::borrow::Cow;

use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::{
    App,
    models::{ModelError, user::User},
    schemas::{ChangePassword, CreateStaffUser, RegisterUser, UpdateProfile},
};
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{
        cleanup_date, cleanup_hashed_token, cleanup_id, cleanup_password, cleanup_uuid,
        cleanup_verification_token,
    },
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

fn update_profile(name: &str, email: &str, image: Option<&str>) -> UpdateProfile<'static> {
    UpdateProfile::new(
        Cow::Owned(name.to_string()),
        Cow::Owned(email.to_string()),
        image.map(|image| Cow::Owned(image.to_string())),
    )
}

fn staff_user_params(email: &str, role_id: i32) -> CreateStaffUser<'static> {
    CreateStaffUser::new(
        Cow::Owned(email.to_string()),
        Cow::Owned("Warehouse Manager".to_string()),
        Cow::Owned("Password123".to_string()),
        Cow::Owned("Password123".to_string()),
        role_id,
        None,
    )
}

#[rstest]
#[case("can_find_support_access", "bd6f7c26-d2c9-487e-b837-8f77be468033")]
#[case("can_find_manager_access", "69768c35-da6d-46cf-bc17-ea78f7e21a6f")]
#[case("can_find_customer_access", "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8")]
#[case(
    "cannot_find_access_for_unknown_user",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn can_find_user_access(#[case] test_name: &str, #[case] user_pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();
    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");

    let result = User::find_access(
        ctx.db(),
        Uuid::parse_str(user_pid).expect("Test user PID must be a UUID"),
    )
    .await;

    assert_debug_snapshot!(test_name, result);
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
        Some(Cow::Owned(
            "https://cdn.example.com/users/test_user.png".to_string(),
        )),
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
        None,
    );

    let result = User::create(ctx.db(), &params).await;

    assert_debug_snapshot!(result);
}

#[rstest]
#[case("can_create_staff_user", "warehouse.manager@silk.com", 11, true)]
#[case(
    "cannot_create_staff_user_when_email_already_exists",
    "john.doe@acme.com",
    11,
    false
)]
#[case(
    "cannot_create_staff_user_with_customer_role",
    "shopper@silk.com",
    22,
    true
)]
#[case(
    "cannot_create_staff_user_with_missing_role",
    "missing.role@silk.com",
    999,
    true
)]
#[tokio::test]
#[serial]
async fn create_staff_user(
    #[case] test_name: &str,
    #[case] email: &str,
    #[case] role_id: i32,
    #[case] remove_after: bool,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();
    App::seed(ctx.db()).await.unwrap();

    let result = User::create_staff(ctx.db(), &staff_user_params(email, role_id)).await;
    let assignments = match &result {
        Ok(user) => service::models::UserRole::find_by_user(ctx.db(), user.id()).await,
        Err(_) => Ok(Vec::new()),
    };
    let assigned_role_ids = assignments
        .as_ref()
        .expect("Failed to inspect the created user's roles")
        .iter()
        .map(service::models::UserRole::role_id)
        .collect::<Vec<_>>();
    let expected_role_ids = if result.is_ok() {
        vec![role_id]
    } else {
        Vec::new()
    };
    assert_eq!(assigned_role_ids, expected_role_ids);

    if remove_after {
        sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(email)
            .execute(ctx.db())
            .await
            .expect("Failed to remove test user");
    }

    with_settings!({
        filters => {
            let mut filters = cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_password());
            filters.extend(cleanup_id());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, (result, assignments))
    })
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
            let mut filters = cleanup_verification_token().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[ignore = "change verification expiry in data/users.json before running"]
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
            cleanup_date().to_vec()
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[tokio::test]
#[serial]
async fn can_set_reset_token() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let mut user = User::find_by_email(ctx.db(), "john.doe@acme.com")
        .await
        .unwrap();

    let token = Uuid::new_v4().to_string();

    let result = user
        .set_reset_token(ctx.db(), &token, ctx.config().auth().refresh_token_expiry())
        .await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_hashed_token());
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[tokio::test]
#[serial]
async fn can_reset_password() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let mut user = User::find_by_email(ctx.db(), "john.doe@acme.com")
        .await
        .unwrap();

    let token = Uuid::new_v4().to_string();

    user.set_reset_token(ctx.db(), &token, ctx.config().auth().refresh_token_expiry())
        .await
        .unwrap();

    let result = User::reset_password(ctx.db(), &token, "Password123").await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_hashed_token());
            filters.extend(cleanup_password());
            filters
        }
    }, {
        assert_debug_snapshot!(result)
    })
}

#[rstest]
#[case(
    "can_update_profile_name_and_email",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    update_profile(
        "John Updated",
        "john.updated@acme.com",
        Some("https://cdn.example.com/users/john_updated.png")
    )
)]
#[case(
    "can_update_profile_with_same_email",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    update_profile("John Renamed", "john.doe@acme.com", None)
)]
#[case(
    "cannot_update_profile_when_email_already_exists",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    update_profile("John Updated", "jane.smith@globex.com", None)
)]
#[case(
    "cannot_update_profile_when_claims_key_is_invalid",
    "not-a-uuid",
    update_profile("John Updated", "john.updated@acme.com", None)
)]
#[case(
    "cannot_update_profile_when_user_does_not_exist",
    "00000000-0000-4000-8000-000000000000",
    update_profile("John Updated", "john.updated@acme.com", None)
)]
#[tokio::test]
#[serial]
async fn can_update_profile(
    #[case] test_name: &str,
    #[case] claims_key: &str,
    #[case] params: UpdateProfile<'static>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let result = User::update_profile(ctx.db(), claims_key, &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_password());
            filters.extend(cleanup_verification_token());
            filters
        }
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[tokio::test]
#[serial]
async fn can_change_password() {
    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let mut user = User::find_by_email(ctx.db(), "john.doe@acme.com")
        .await
        .unwrap();
    let token = Uuid::new_v4().to_string();

    user.set_reset_token(ctx.db(), &token, ctx.config().auth().refresh_token_expiry())
        .await
        .unwrap();

    let params = ChangePassword::new(
        Cow::Borrowed("Password"),
        Cow::Borrowed("NewPassword123"),
        Cow::Borrowed("NewPassword123"),
    );
    let changed = User::change_password(ctx.db(), &user.pid().to_string(), &params)
        .await
        .unwrap();

    assert_eq!(changed.pid(), user.pid());
    assert_ne!(changed.password_hash(), user.password_hash());
    assert!(changed.reset_token_hash().is_none());
    assert!(changed.reset_token_expires_at().is_none());
    assert!(changed.verify_password("NewPassword123").is_ok());
    assert!(matches!(
        changed.verify_password("Password"),
        Err(ModelError::InvalidCredentials)
    ));
}

#[tokio::test]
#[serial]
async fn cannot_change_password_when_current_password_is_wrong() {
    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let user = User::find_by_email(ctx.db(), "john.doe@acme.com")
        .await
        .unwrap();

    let params = ChangePassword::new(
        Cow::Borrowed("WrongPassword"),
        Cow::Borrowed("NewPassword123"),
        Cow::Borrowed("NewPassword123"),
    );
    let result = User::change_password(ctx.db(), &user.pid().to_string(), &params).await;

    assert!(matches!(result, Err(ModelError::InvalidCredentials)));

    let unchanged = User::find_by_email(ctx.db(), "john.doe@acme.com")
        .await
        .unwrap();

    assert_eq!(unchanged.password_hash(), user.password_hash());
    assert!(unchanged.verify_password("Password").is_ok());
}
