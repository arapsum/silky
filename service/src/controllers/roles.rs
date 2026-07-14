use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::RbacLayer,
    models::{Role, RolePermission},
    schemas::{AssignPermission, NewRole, UpdateRole, Validator},
    utils::{AppJson, AppPath},
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<NewRole<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let new_role = Role::create(ctx.db(), validated).await?;
    let new_role = Role::find_with_users_by_pid(ctx.db(), new_role.pid()).await?;

    Ok((StatusCode::CREATED, Json(new_role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(State(ctx): State<AppState>) -> Result<Response> {
    let roles = Role::find_list_with_users(ctx.db()).await?;

    Ok((StatusCode::OK, Json(roles)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn update(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<UpdateRole<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let updated_role = Role::update(ctx.db(), pid, validated).await?;
    let updated_role = Role::find_with_users_by_pid(ctx.db(), updated_role.pid()).await?;

    Ok((StatusCode::CREATED, Json(updated_role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, AppPath(pid): AppPath<Uuid>) -> Result<Response> {
    let role = Role::find_with_users_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn assign_permission(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<AssignPermission>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let role_permission = RolePermission::assign_permission(ctx.db(), validated).await?;

    Ok((StatusCode::CREATED, Json(role_permission)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn revoke_permission(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<AssignPermission>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    RolePermission::revoke_permission(ctx.db(), validated).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{pid}", get(one).patch(update))
        .route(
            "/permissions",
            post(assign_permission).delete(revoke_permission),
        )
        .route_layer(RbacLayer::for_routes(
            ctx.clone(),
            [
                (
                    Method::POST,
                    Some("/permissions"),
                    permissions::roles::UPDATE,
                ),
                (Method::DELETE, None, permissions::roles::UPDATE),
                (Method::PATCH, None, permissions::roles::UPDATE),
                (Method::GET, None, permissions::roles::READ),
                (Method::POST, None, permissions::roles::CREATE),
            ],
        ))
        .with_state(ctx.clone())
}
