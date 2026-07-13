use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    access_control::permissions,
    middlewares::RbacLayer,
    models::{User, UserRole},
    schemas::{AssignRole, CreateStaffUser, UserListQuery, Validator},
    utils::{AppJson, AppQuery},
    views::AuthResponse,
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

/// Provisions a staff account with its first non-customer role.
///
/// Public customer registration remains under `/auth/register`; this endpoint
/// is protected by the dedicated `users:create` permission instead.
#[tracing::instrument(skip(ctx, params))]
#[debug_handler]
async fn create(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<CreateStaffUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let mut created = User::create_staff(ctx.db(), validated).await?;
    let verification_token = Uuid::new_v4().to_string();

    created
        .set_verification_token(
            ctx.db(),
            &verification_token,
            ctx.config().auth().verification_token_expiry(),
        )
        .await?;

    if let Some(queue) = ctx.queue().get() {
        queue
            .enqueue_welcome(created.pid(), verification_token)
            .await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse::new(
            "Staff account created successfully. Please ask the user to verify their email.",
        )),
    )
        .into_response())
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
            "/",
            post(create).layer(RbacLayer::new(ctx.clone(), permissions::users::CREATE)),
        )
        .route(
            "/roles",
            post(assign_role)
                .delete(revoke_role)
                .layer(RbacLayer::new(ctx.clone(), permissions::users::UPDATE)),
        )
        .with_state(ctx.clone())
}
