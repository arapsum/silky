use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serde_json::json;
use serial_test::serial;
use service::{
    models::{Role, User, UserRole},
    schemas::AssignRole,
};

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("user_roles");
        settings.set_snapshot_path("snapshots/user_roles");
        let _guard = settings.bind_to_scope();
    };
}

fn assign_role(user_id: i32, role_id: i32) -> AssignRole {
    serde_json::from_value(json!({
        "userId": user_id,
        "roleId": role_id
    }))
    .expect("Failed to parse role assignment")
}

fn redactions() -> Vec<(&'static str, &'static str)> {
    let mut filters = cleanup_uuid().to_vec();
    filters.extend(cleanup_date().to_vec());
    filters.extend(cleanup_id());
    filters
}

async fn seed_users_and_roles(db: &sqlx::PgPool) {
    User::seed_data(db, "users.json")
        .await
        .expect("Failed to seed users");
    Role::seed_data(db, "roles.json")
        .await
        .expect("Failed to seed roles");
}

async fn seed_user_roles(db: &sqlx::PgPool) {
    seed_users_and_roles(db).await;
    UserRole::seed_data(db, "userRoles.json")
        .await
        .expect("Failed to seed user roles");
}

#[rstest]
#[case("can_seed_user_roles_from_json", "userRoles.json")]
#[case(
    "when_seed_file_does_not_exist_user_role_seeding_fails",
    "missing-userRoles.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_user_roles(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_users_and_roles(ctx.db()).await;

    let result = UserRole::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_assign_role_to_user", 22, 11)]
#[tokio::test]
#[serial]
async fn can_assign_role_to_user(
    #[case] test_name: &str,
    #[case] user_id: i32,
    #[case] role_id: i32,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_user_roles(ctx.db()).await;

    let params = assign_role(user_id, role_id);
    let result = UserRole::assign_role(ctx.db(), &params).await;

    with_settings!({
        filters => redactions()
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case("cannot_assign_role_when_user_already_has_role", 11, 22)]
#[tokio::test]
#[serial]
async fn cannot_assign_duplicate_role_to_user(
    #[case] test_name: &str,
    #[case] user_id: i32,
    #[case] role_id: i32,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_user_roles(ctx.db()).await;

    let params = assign_role(user_id, role_id);
    let result = UserRole::assign_role(ctx.db(), &params).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_user_roles_by_user", 11)]
#[case("can_find_user_roles_by_user_when_none_exist", 22)]
#[tokio::test]
#[serial]
async fn can_find_user_roles_by_user(#[case] test_name: &str, #[case] user_id: i32) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_user_roles(ctx.db()).await;

    let result = UserRole::find_by_user(ctx.db(), user_id).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_user_roles_by_role", 11)]
#[case("can_find_user_roles_by_role_when_none_exist", 999)]
#[tokio::test]
#[serial]
async fn can_find_user_roles_by_role(#[case] test_name: &str, #[case] role_id: i32) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_user_roles(ctx.db()).await;

    let result = UserRole::find_by_role(ctx.db(), role_id).await;

    assert_debug_snapshot!(test_name, result);
}
