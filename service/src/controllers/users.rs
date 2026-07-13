use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::RbacLayer,
    models::{User, UserRole},
    schemas::{AssignRole, UserListQuery, Validator},
    utils::{AppJson, AppQuery},
};

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(
    State(ctx): State<AppState>,
    AppQuery(query): AppQuery<UserListQuery>,
) -> Result<Response> {
    let validator = Validator::new(query);
    let validated = validator.validate()?;

    let users = User::find_list_with_roles(ctx.db(), validated.role()).await?;

    Ok((StatusCode::OK, Json(users)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn assign_role(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<AssignRole>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user_role = UserRole::assign_role(ctx.db(), validated).await?;

    Ok((StatusCode::CREATED, Json(user_role)).into_response())
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn revoke_role(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<AssignRole>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    UserRole::revoke_role(ctx.db(), validated).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(list).layer(RbacLayer::new(ctx.clone(), permissions::users::READ)),
        )
        .route(
            "/roles",
            post(assign_role)
                .delete(revoke_role)
                .layer(RbacLayer::new(ctx.clone(), permissions::users::UPDATE)),
        )
        .with_state(ctx.clone())
}
