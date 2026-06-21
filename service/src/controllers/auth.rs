use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use uuid::Uuid;

use crate::{
    AppState, Result,
    models::User,
    schemas::{RegisterUser, Validator},
    utils::AppJson,
    views::AuthResponse,
};

async fn register(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<RegisterUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let mut created = User::create(ctx.db(), validated).await?;

    let verification_token = Uuid::new_v4().to_string();

    created
        .set_verification_token(
            ctx.db(),
            &verification_token,
            ctx.config().auth().verification_token_expiry(),
        )
        .await?;

    tracing::info!("Send verification token {verification_token} to user's email");

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse::new(
            "User registered successfully. Please check your email to verify your account.",
        )),
    )
        .into_response())
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .with_state(ctx.clone())
}
