use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serde_json::{Value, json};
use serial_test::serial;
use service::{
    models::{Permission, Role, RolePermission},
    schemas::{AssignPermission, PermissionRoleQuery},
};
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_id, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("roles_permissions");
        settings.set_snapshot_path("snapshots/roles_permissions");
        let _guard = settings.bind_to_scope();
    };
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

fn assign_permission(role_id: i32, permission_id: i32) -> AssignPermission {
    serde_json::from_value(json!({
        "roleId": role_id,
        "permissionId": permission_id
    }))
    .expect("Failed to parse permission assignment")
}

fn permission_role_query(role_id: Option<i32>, permission_id: Option<i32>) -> PermissionRoleQuery {
    let mut value = serde_json::Map::new();

    if let Some(role_id) = role_id {
        value.insert("role_id".to_string(), json!(role_id));
    }

    if let Some(permission_id) = permission_id {
        value.insert("permission_id".to_string(), json!(permission_id));
    }

    serde_json::from_value(Value::Object(value)).expect("Failed to parse role permission query")
}

fn redactions() -> Vec<(&'static str, &'static str)> {
    let mut filters = cleanup_uuid().to_vec();
    filters.extend(cleanup_date().to_vec());
    filters.extend(cleanup_id());
    filters
}

async fn seed_roles_and_permissions(db: &sqlx::PgPool) {
    Role::seed_data(db, "roles.json")
        .await
        .expect("Failed to seed roles");
    Permission::seed_data(db, "permissions.json")
        .await
        .expect("Failed to seed permissions");
}

async fn seed_role_permissions(db: &sqlx::PgPool) {
    seed_roles_and_permissions(db).await;
    RolePermission::seed_data(db, "rolesPermission.json")
        .await
        .expect("Failed to seed role permissions");
}

#[rstest]
#[case("can_seed_role_permissions_from_json", "rolesPermission.json")]
#[case(
    "when_seed_file_does_not_exist_role_permission_seeding_fails",
    "missing-rolesPermission.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_role_permissions(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_roles_and_permissions(ctx.db()).await;

    let result = RolePermission::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_seed_role_permissions_idempotently")]
#[tokio::test]
#[serial]
async fn can_seed_role_permissions_idempotently(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let result = RolePermission::seed_data(ctx.db(), "rolesPermission.json").await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_assign_permission_to_role", 22, 114)]
#[tokio::test]
#[serial]
async fn can_assign_permission_to_role(
    #[case] test_name: &str,
    #[case] role_id: i32,
    #[case] permission_id: i32,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let params = assign_permission(role_id, permission_id);
    let result = RolePermission::assign_permission(ctx.db(), &params).await;

    with_settings!({
        filters => redactions()
    }, {
        assert_debug_snapshot!(test_name, result)
    })
}

#[rstest]
#[case("cannot_assign_permission_when_role_already_has_permission", 22, 113)]
#[tokio::test]
#[serial]
async fn cannot_assign_duplicate_permission_to_role(
    #[case] test_name: &str,
    #[case] role_id: i32,
    #[case] permission_id: i32,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let params = assign_permission(role_id, permission_id);
    let result = RolePermission::assign_permission(ctx.db(), &params).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_all_role_permissions", None, None)]
#[case("can_find_role_permissions_by_role", Some(11), None)]
#[case("can_find_role_permissions_by_permission", None, Some(113))]
#[case(
    "can_find_role_permissions_by_role_and_permission",
    Some(11),
    Some(106)
)]
#[case("can_find_role_permissions_when_none_exist", Some(22), Some(116))]
#[tokio::test]
#[serial]
async fn can_find_role_permissions(
    #[case] test_name: &str,
    #[case] role_id: Option<i32>,
    #[case] permission_id: Option<i32>,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let result =
        RolePermission::find_all(ctx.db(), permission_role_query(role_id, permission_id)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_find_role_permission_by_pid",
    "d1cb710f-f36b-49f8-b04c-f18aa0393c79"
)]
#[tokio::test]
#[serial]
async fn can_find_role_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let result = RolePermission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "cannot_find_role_permission_when_pid_does_not_exist",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn cannot_find_role_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    seed_role_permissions(ctx.db()).await;

    let result = RolePermission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}
