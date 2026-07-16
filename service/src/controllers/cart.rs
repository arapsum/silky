use axum::{Json, Router, debug_handler, extract::State, http::StatusCode, routing::post};

use crate::{
    AppState, Result,
    models::CartQuote,
    schemas::{CartQuoteRequest, Validator},
    utils::AppJson,
};

#[tracing::instrument(skip(ctx, params))]
#[debug_handler]
async fn quote(
    State(ctx): State<AppState>,
    AppJson(params): AppJson<CartQuoteRequest>,
) -> Result<(StatusCode, Json<CartQuote>)> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;
    let quote = CartQuote::create(ctx.db(), validated).await?;
    Ok((StatusCode::OK, Json(quote)))
}

pub fn router(ctx: &AppState) -> Router {
    Router::new()
        .route("/quote", post(quote))
        .with_state(ctx.clone())
}
