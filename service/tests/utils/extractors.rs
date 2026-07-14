use axum::body::Bytes;
use axum::{
    Json, Router,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_test::TestServer;
use serde::Deserialize;
use serde_json::json;
use service::{
    Result,
    utils::{AppExtension, AppJson, AppPath, AppQuery},
};
use uuid::Uuid;

#[derive(Clone)]
struct TestExtension;

#[derive(Deserialize)]
struct TestQuery {
    page: u32,
}

#[derive(Deserialize)]
struct TestPayload {
    name: String,
}

#[tokio::test]
async fn app_json_extracts_valid_json_body() {
    let server = TestServer::new(router());

    let response = server
        .post("/json")
        .json(&json!({ "name": "extractor" }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(
        response.json::<serde_json::Value>(),
        json!({ "name": "extractor" })
    );
}

#[tokio::test]
async fn app_json_rejection_uses_application_error_shape() {
    let server = TestServer::new(router());

    let response = server
        .post("/json")
        .add_header(CONTENT_TYPE, "application/json")
        .bytes(Bytes::from_static(b"{"))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert!(response.text().starts_with("{\"error\":"));
}

#[tokio::test]
async fn app_json_missing_content_type_uses_application_error_shape() {
    let server = TestServer::new(router());

    let response = server.post("/json").text("{}").await;

    assert_eq!(response.status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(response.text().starts_with("{\"error\":"));
}

#[tokio::test]
async fn app_path_rejection_uses_application_error_shape() {
    let server = TestServer::new(router());

    let response = server.get("/path/not-a-uuid").await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json::<serde_json::Value>(),
        json!({
            "error": "The supplied resource identifier is invalid.",
            "code": "invalid_path_parameter",
            "field": "id",
        })
    );
}

#[tokio::test]
async fn app_query_rejection_uses_application_error_shape() {
    let server = TestServer::new(router());

    let response = server.get("/query?page=not-a-number").await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert!(response.text().starts_with("{\"error\":"));
}

#[tokio::test]
async fn app_extension_rejection_uses_application_error_shape() {
    let server = TestServer::new(router());

    let response = server.get("/extension").await;

    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.text().starts_with("{\"error\":"));
}

async fn path(AppPath(id): AppPath<Uuid>) -> Result<Response> {
    Ok((StatusCode::OK, Json(json!({ "id": id }))).into_response())
}

async fn json(AppJson(payload): AppJson<TestPayload>) -> Result<Response> {
    Ok((StatusCode::OK, Json(json!({ "name": payload.name }))).into_response())
}

async fn query(AppQuery(query): AppQuery<TestQuery>) -> Result<Response> {
    Ok((StatusCode::OK, Json(json!({ "page": query.page }))).into_response())
}

async fn extension(AppExtension(_extension): AppExtension<TestExtension>) -> Result<Response> {
    Ok(StatusCode::OK.into_response())
}

fn router() -> Router {
    Router::new()
        .route("/json", post(json))
        .route("/path/{id}", get(path))
        .route("/query", get(query))
        .route("/extension", get(extension))
}
