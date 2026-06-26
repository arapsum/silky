use insta::{Settings, assert_debug_snapshot};
use rstest::rstest;
use serial_test::serial;
use service::models::Permission;
use uuid::Uuid;

use crate::boot_test;

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("permissions");
        settings.set_snapshot_path("snapshots/permissions");
        let _guard = settings.bind_to_scope();
    };
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("Failed to parse UUID")
}

async fn grant_permission(db: &sqlx::PgPool, role: &str, permission: &str) {
    sqlx::query(
        r"
        INSERT INTO roles_permissions (role_id, permission_id)
        SELECT roles.id, permissions.id
        FROM roles
        CROSS JOIN permissions
        WHERE roles.name = $1
            AND permissions.name = $2
        ON CONFLICT (role_id, permission_id) DO NOTHING
    ",
    )
    .bind(role)
    .bind(permission)
    .execute(db)
    .await
    .expect("Failed to grant permission");
}

async fn assign_role(db: &sqlx::PgPool, email: &str, role: &str) {
    sqlx::query(
        r"
        INSERT INTO users_roles (user_id, role_id)
        SELECT users.id, roles.id
        FROM users
        CROSS JOIN roles
        WHERE users.email = $1
            AND roles.name = $2
        ON CONFLICT (user_id, role_id) DO NOTHING
    ",
    )
    .bind(email)
    .bind(role)
    .execute(db)
    .await
    .expect("Failed to assign role");
}

#[rstest]
#[case("can_seed_permissions_from_json", "permissions.json")]
#[case(
    "when_seed_file_does_not_exist_permission_seeding_fails",
    "missing-permissions.json"
)]
#[tokio::test]
#[serial]
async fn can_seed_permissions(#[case] test_name: &str, #[case] file: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let result = Permission::seed_data(ctx.db(), file).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_find_roles_read_permission_by_pid",
    "9e230b11-cb47-4fe8-8bc0-5185fd9f9bb2"
)]
#[case(
    "can_find_roles_create_permission_by_pid",
    "3fdbf302-c05a-4f58-82d8-1f7efa9ea8a6"
)]
#[case(
    "can_find_permissions_read_permission_by_pid",
    "a0199d51-0147-477f-8778-070785ee81f3"
)]
#[tokio::test]
#[serial]
async fn can_find_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "cannot_find_permission_when_pid_does_not_exist",
    "00000000-0000-0000-0000-000000000000"
)]
#[tokio::test]
#[serial]
async fn cannot_find_permission_by_pid(#[case] test_name: &str, #[case] pid: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_by_pid(ctx.db(), uuid(pid)).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_permission_list")]
#[tokio::test]
#[serial]
async fn can_find_permission_list(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    Permission::seed_data(ctx.db(), "permissions.json")
        .await
        .expect("Failed to seed permissions");

    let result = Permission::find_list(ctx.db(), None).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case("can_find_permission_list_by_role")]
#[tokio::test]
#[serial]
async fn can_find_permission_list_by_role(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");
    grant_permission(ctx.db(), "customer", "roles:read").await;
    grant_permission(ctx.db(), "customer", "permissions:read").await;
    grant_permission(ctx.db(), "administrator", "roles:create").await;

    let result = Permission::find_list(ctx.db(), Some(" Customer ")).await;

    assert_debug_snapshot!(test_name, result);
}

#[rstest]
#[case(
    "can_grant_permission_when_customer_role_has_permission",
    "customer",
    "roles:read",
    true,
    true,
    true
)]
#[case(
    "cannot_grant_permission_when_assigned_role_lacks_permission",
    "customer",
    "roles:write",
    false,
    true,
    false
)]
#[case(
    "cannot_grant_permission_when_user_has_no_role",
    "administrator",
    "roles:read",
    true,
    false,
    false
)]
#[tokio::test]
#[serial]
async fn can_check_permission_grants_for_user_roles(
    #[case] test_name: &str,
    #[case] role: &str,
    #[case] permission: &str,
    #[case] grant_role_permission: bool,
    #[case] assign_user_role: bool,
    #[case] expected: bool,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");

    if grant_role_permission {
        grant_permission(ctx.db(), role, permission).await;
    }

    if assign_user_role {
        assign_role(ctx.db(), "john.doe@acme.com", role).await;
    }

    let result = Permission::is_granted_to_user_role(
        ctx.db(),
        uuid("bd6f7c26-d2c9-487e-b837-8f77be468033"),
        permission,
    )
    .await;

    assert!(matches!(&result, Ok(value) if *value == expected));
    assert_debug_snapshot!(test_name, result);
}
