use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::RbacLayer,
    models::Permission,
    schemas::{PermissionListQuery, Validator},
    utils::{AppPath, AppQuery},
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppQuery(query): AppQuery<PermissionListQuery>,
) -> Result<Response> {
    let validator = Validator::new(query);
    let validated = validator.validate()?;

    let permissions = Permission::find_list(ctx.db(), validated.role()).await?;

    Ok((StatusCode::OK, Json(permissions)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn one(State(ctx): State<AppState>, AppPath(pid): AppPath<Uuid>) -> Result<Response> {
    let permission = Permission::find_by_pid(ctx.db(), pid).await?;

    Ok((StatusCode::OK, Json(permission)).into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(list).layer(RbacLayer::new(
                ctx.clone(),
                permissions::permission_records::READ,
            )),
        )
        .route(
            "/{pid}",
            get(one).layer(RbacLayer::new(
                ctx.clone(),
                permissions::permission_records::READ,
            )),
        )
        .with_state(ctx.clone())
}
