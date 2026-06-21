use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::post,
};
use axum_extra::extract::cookie;
use serde_json::json;
use uuid::Uuid;

use crate::{
    AppState, Result,
    models::User,
    schemas::{LoginUser, RegisterUser, Validator},
    utils::AppJson,
    views::{AuthResponse, LoginResponse},
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

    // TODO: Send verification token to user's email
    tracing::info!("Send verification token {verification_token} to user's email");

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse::new(
            "User registered successfully. Please check your email to verify your account.",
        )),
    )
        .into_response())
}

async fn login(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<LoginUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user = User::find_by_email(ctx.db(), validated.email()).await?;
    user.verify_password(validated.password())?;

    let sub = user.pid().to_string();
    let access_token = ctx.auth().access().generate_token(&sub)?;
    let refresh_token = ctx.auth().refresh().generate_token(&sub)?;

    let access_cookie = cookie::Cookie::build(("access_token", &access_token))
        .path("/")
        .http_only(false)
        .max_age(time::Duration::seconds(ctx.auth().access().expires_in()))
        .same_site(cookie::SameSite::Lax)
        .secure(false);

    let refresh_cookie = cookie::Cookie::build(("refresh_token", &refresh_token))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(ctx.auth().refresh().expires_in()))
        .same_site(cookie::SameSite::Lax)
        .secure(false);

    let mut response = Response::builder().status(StatusCode::OK).body(Body::from(
        json!(LoginResponse::new(&user, &access_token)).to_string(),
    ))?;

    response.headers_mut().append(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access_token}"))?,
    );
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(access_cookie.to_string().as_str())?,
    );
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(refresh_cookie.to_string().as_str())?,
    );

    Ok(response)
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(ctx.clone())
}
