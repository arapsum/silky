use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};

use crate::{
    AppState, Result,
    middlewares::RbacLayer,
    models::User,
    schemas::{UserListQuery, Validator},
    utils::AppQuery,
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

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(list).layer(RbacLayer::new(ctx.clone(), "users:read")),
        )
        .with_state(ctx.clone())
}
