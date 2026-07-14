use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;
use service::models::Permission;
use uuid::Uuid;

use crate::{boot_test, utils::cleanup_date};

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
#[case(
    "can_find_admin_access_permission_by_pid",
    "dbd690cc-58dd-471b-93b6-7dbd3896d3da"
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

    with_settings!({ filters => cleanup_date().to_vec() }, {
        assert_debug_snapshot!(test_name, result)
    });
}

#[rstest]
#[case("can_find_all_permissions_for_administrator")]
#[tokio::test]
#[serial]
async fn can_find_all_permissions_for_administrator(#[case] test_name: &str) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");

    let result = Permission::find_list(ctx.db(), Some(" Administrator ")).await;

    assert!(matches!(&result, Ok(permissions) if permissions.len() == 27));
    with_settings!({ filters => cleanup_date().to_vec() }, {
        assert_debug_snapshot!(test_name, result)
    });
}

#[rstest]
#[case(
    "permission_is_granted_through_customer_role",
    "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8",
    "categories:read",
    true
)]
#[case(
    "permission_is_not_granted_when_customer_role_lacks_it",
    "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8",
    "roles:read",
    false
)]
#[case(
    "permission_is_not_granted_when_user_does_not_exist",
    "00000000-0000-0000-0000-000000000000",
    "categories:read",
    false
)]
#[case(
    "manager_can_delete_products",
    "69768c35-da6d-46cf-bc17-ea78f7e21a6f",
    "products:delete",
    true
)]
#[case(
    "manager_cannot_update_roles",
    "69768c35-da6d-46cf-bc17-ea78f7e21a6f",
    "roles:update",
    false
)]
#[case(
    "support_can_update_orders",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    "orders:update",
    true
)]
#[case(
    "support_cannot_update_products",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    "products:update",
    false
)]
#[case(
    "manager_can_access_admin_application",
    "69768c35-da6d-46cf-bc17-ea78f7e21a6f",
    "admin:access",
    true
)]
#[case(
    "support_can_access_admin_application",
    "bd6f7c26-d2c9-487e-b837-8f77be468033",
    "admin:access",
    true
)]
#[case(
    "customer_cannot_access_admin_application",
    "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8",
    "admin:access",
    false
)]
#[tokio::test]
#[serial]
async fn can_check_permission_grants_for_user_roles(
    #[case] test_name: &str,
    #[case] user_pid: &str,
    #[case] permission: &str,
    #[case] expected: bool,
) {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db())
        .await
        .expect("Failed to seed data");

    let result = Permission::is_granted_to_user_role(ctx.db(), uuid(user_pid), permission).await;

    assert!(matches!(&result, Ok(value) if *value == expected));
    assert_debug_snapshot!(test_name, result);
}
