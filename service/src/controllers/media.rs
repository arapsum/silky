use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use sha1::{Digest, Sha1};
use uuid::Uuid;

use crate::{
    AppState, Error, Result,
    access_control::permissions,
    context::Claims,
    middlewares::{AuthLayer, RbacLayer},
    models::{FinalizeMediaAsset, MediaAsset},
    utils::{AppExtension, AppJson, AppPath},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignUploadRequest {
    kind: String,
    checksum: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SignUploadResponse {
    asset_pid: Uuid,
    cloud_name: String,
    api_key: String,
    timestamp: i64,
    folder: String,
    public_id: String,
    signature: String,
}

fn folder_for_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "user" => Some("silk/users"),
        "category" => Some("silk/categories"),
        "product" => Some("silk/products"),
        _ => None,
    }
}

fn sign_params(folder: &str, public_id: &str, timestamp: i64, secret: &str) -> String {
    let payload = format!("folder={folder}&public_id={public_id}&timestamp={timestamp}");
    let mut hasher = Sha1::new();
    hasher.update(payload.as_bytes());
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

async fn create_upload_signature(
    ctx: &AppState,
    claims: &Claims,
    folder: &str,
    checksum: Option<&str>,
) -> Result<SignUploadResponse> {
    let cloudinary = ctx
        .config()
        .cloudinary()
        .ok_or_else(|| Error::ValidationError("Cloudinary is not configured".to_string()))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::InvalidToken)?
        .as_secs()
        .try_into()
        .map_err(|_| Error::InvalidToken)?;
    let public_id = Uuid::new_v4().to_string();
    let user_pid = Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden)?;
    let asset =
        MediaAsset::create_pending(ctx.db(), user_pid, &public_id, folder, checksum).await?;
    let signature = sign_params(folder, &public_id, timestamp, cloudinary.api_secret());

    Ok(SignUploadResponse {
        asset_pid: asset.pid(),
        cloud_name: cloudinary.cloud_name().to_string(),
        api_key: cloudinary.api_key().to_string(),
        timestamp,
        folder: folder.to_string(),
        public_id,
        signature,
    })
}

#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn sign_upload(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppJson(params): AppJson<SignUploadRequest>,
) -> Result<impl IntoResponse> {
    let folder = folder_for_kind(params.kind.trim())
        .ok_or_else(|| Error::ValidationError("Unsupported media kind".to_string()))?;
    Ok((
        StatusCode::OK,
        Json(create_upload_signature(&ctx, &claims, folder, params.checksum.as_deref()).await?),
    ))
}

/// Signs an avatar upload for the authenticated customer without granting
/// catalogue-media permissions.
#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn sign_profile_upload(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppJson(params): AppJson<SignUploadRequest>,
) -> Result<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(
            create_upload_signature(&ctx, &claims, "silk/users", params.checksum.as_deref())
                .await?,
        ),
    ))
}

#[tracing::instrument(skip(ctx, params))]
#[debug_handler]
async fn finalize(
    State(ctx): State<AppState>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<FinalizeMediaAsset>,
) -> Result<impl IntoResponse> {
    let cloudinary = ctx
        .config()
        .cloudinary()
        .ok_or_else(|| Error::ValidationError("Cloudinary is not configured".to_string()))?;
    let expected_prefix = format!("https://res.cloudinary.com/{}/", cloudinary.cloud_name());
    if !params.secure_url.starts_with(&expected_prefix) {
        return Err(Error::ValidationError("Invalid Cloudinary asset URL".to_string()).into());
    }

    let asset = MediaAsset::finalize(ctx.db(), pid, &params).await?;
    Ok((StatusCode::OK, Json(asset)))
}

/// Finalizes an avatar upload owned by the current customer.
#[tracing::instrument(skip(ctx, claims, params))]
#[debug_handler]
async fn finalize_profile_upload(
    State(ctx): State<AppState>,
    AppExtension(claims): AppExtension<Claims>,
    AppPath(pid): AppPath<Uuid>,
    AppJson(params): AppJson<FinalizeMediaAsset>,
) -> Result<impl IntoResponse> {
    let cloudinary = ctx
        .config()
        .cloudinary()
        .ok_or_else(|| Error::ValidationError("Cloudinary is not configured".to_string()))?;
    let expected_prefix = format!("https://res.cloudinary.com/{}/", cloudinary.cloud_name());
    if !params.secure_url.starts_with(&expected_prefix) {
        return Err(Error::ValidationError("Invalid Cloudinary asset URL".to_string()).into());
    }

    let user_pid = Uuid::parse_str(claims.sub()).map_err(|_| Error::Forbidden)?;
    let asset = MediaAsset::finalize_owned(ctx.db(), pid, user_pid, &params).await?;
    Ok((StatusCode::OK, Json(asset)))
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn list(State(ctx): State<AppState>) -> Result<impl IntoResponse> {
    Ok((StatusCode::OK, Json(MediaAsset::list(ctx.db()).await?)))
}

#[tracing::instrument(skip(ctx))]
#[debug_handler]
async fn reconcile(State(ctx): State<AppState>) -> Result<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(MediaAsset::quarantine_orphans(ctx.db()).await?),
    ))
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(list).layer(RbacLayer::new(ctx.clone(), permissions::media::READ)),
        )
        .route(
            "/reconcile",
            post(reconcile).layer(RbacLayer::new(ctx.clone(), permissions::media::DELETE)),
        )
        .route(
            "/sign",
            post(sign_upload).layer(RbacLayer::new(ctx.clone(), permissions::media::CREATE)),
        )
        .route("/profile/sign", post(sign_profile_upload))
        .route(
            "/{pid}/finalize",
            put(finalize).layer(RbacLayer::new(ctx.clone(), permissions::media::UPDATE)),
        )
        .route("/profile/{pid}/finalize", put(finalize_profile_upload))
        .with_state(ctx.clone())
        .layer(AuthLayer::new(ctx.clone()))
}
