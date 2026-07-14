use std::{env, ffi::OsString};

use axum::http::{HeaderValue, StatusCode};
use axum_test::TestServer;
use serde_json::{Value, json};
use serial_test::serial;
use sha1::{Digest, Sha1};

use crate::utils;

const TEST_CLOUD_NAME: &str = "test-cloud";
const TEST_API_KEY: &str = "test-api-key";
const TEST_API_SECRET: &str = "test-api-secret";

struct CloudinaryEnvGuard {
    previous: Vec<(&'static str, Option<OsString>)>,
}

impl CloudinaryEnvGuard {
    fn set() -> Self {
        let variables = [
            ("APP_CLOUDINARY_CLOUD_NAME", TEST_CLOUD_NAME),
            ("APP_CLOUDINARY_API_KEY", TEST_API_KEY),
            ("APP_CLOUDINARY_API_SECRET", TEST_API_SECRET),
        ];
        let previous = variables
            .iter()
            .map(|(key, _)| (*key, env::var_os(key)))
            .collect();

        for (key, value) in variables {
            // SAFETY: Media request tests are serial and restore variables on drop.
            unsafe { env::set_var(key, value) };
        }

        Self { previous }
    }
}

impl Drop for CloudinaryEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            // SAFETY: Restore the process environment modified by this serial test.
            unsafe {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }
}

async fn access_token(server: &TestServer) -> HeaderValue {
    utils::login_users(
        server,
        &json!({"email": "admin@silk.com", "password": "Password"}),
    )
    .await
    .access_token
}

fn auth_header(token: HeaderValue) -> (axum::http::HeaderName, HeaderValue) {
    utils::auth_header(token)
}

fn expected_signature(payload: &Value) -> String {
    let canonical = format!(
        "folder={}&public_id={}&timestamp={}",
        payload["folder"].as_str().unwrap(),
        payload["publicId"].as_str().unwrap(),
        payload["timestamp"].as_i64().unwrap()
    );
    let mut hasher = Sha1::new();
    hasher.update(canonical.as_bytes());
    hasher.update(TEST_API_SECRET.as_bytes());
    hex::encode(hasher.finalize())
}

#[tokio::test]
#[serial]
async fn sign_finalize_list_and_reconcile_media_assets() {
    let _cloudinary = CloudinaryEnvGuard::set();

    crate::request(|server, ctx| async move {
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let token = access_token(&server).await;
        let (header, value) = auth_header(token.clone());
        let sign_response = server
            .post("/media/sign")
            .add_header(header, value)
            .json(&json!({"kind": "product", "checksum": "test-checksum"}))
            .await;

        assert_eq!(sign_response.status_code(), StatusCode::OK);
        let signed: Value = sign_response.json();
        assert_eq!(signed["cloudName"], json!(TEST_CLOUD_NAME));
        assert_eq!(signed["apiKey"], json!(TEST_API_KEY));
        assert_eq!(signed["folder"], json!("silk/products"));
        assert!(uuid::Uuid::parse_str(signed["publicId"].as_str().unwrap()).is_ok());
        assert_eq!(signed["signature"], json!(expected_signature(&signed)));

        let asset_pid = signed["assetPid"].as_str().unwrap();
        let public_id = format!(
            "{}/{}",
            signed["folder"].as_str().unwrap(),
            signed["publicId"].as_str().unwrap()
        );
        let secure_url =
            format!("https://res.cloudinary.com/{TEST_CLOUD_NAME}/image/upload/{public_id}.png");

        let (header, value) = auth_header(token.clone());
        let invalid_finalize = server
            .put(&format!("/media/{asset_pid}/finalize"))
            .add_header(header, value)
            .json(&json!({
                "publicId": public_id,
                "secureUrl": "https://cdn.example.com/not-cloudinary.png"
            }))
            .await;
        assert_eq!(invalid_finalize.status_code(), StatusCode::BAD_REQUEST);

        let (header, value) = auth_header(token.clone());
        let finalize_response = server
            .put(&format!("/media/{asset_pid}/finalize"))
            .add_header(header, value)
            .json(&json!({
                "publicId": public_id,
                "assetId": "cloudinary-asset-id",
                "secureUrl": secure_url,
                "format": "png",
                "bytes": 1024,
                "width": 640,
                "height": 480,
                "checksum": "test-checksum"
            }))
            .await;
        assert_eq!(finalize_response.status_code(), StatusCode::OK);
        let finalized: Value = finalize_response.json();
        assert_eq!(finalized["status"], json!("active"));
        assert_eq!(finalized["publicId"], json!(public_id));

        let (header, value) = auth_header(token.clone());
        let list_response = server.get("/media").add_header(header, value).await;
        assert_eq!(list_response.status_code(), StatusCode::OK);
        let assets: Value = list_response.json();
        assert!(
            assets
                .as_array()
                .unwrap()
                .iter()
                .any(|asset| asset["pid"] == json!(asset_pid))
        );

        let (header, value) = auth_header(token);
        let reconcile_response = server
            .post("/media/reconcile")
            .add_header(header, value)
            .await;
        assert_eq!(reconcile_response.status_code(), StatusCode::OK);
        let quarantined: Value = reconcile_response.json();
        assert!(
            quarantined
                .as_array()
                .unwrap()
                .iter()
                .any(|asset| asset["pid"] == json!(asset_pid) && asset["status"] == "quarantined")
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn media_endpoints_require_authentication() {
    crate::request(|server, ctx| async move {
        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server
            .post("/media/sign")
            .json(&json!({"kind": "product"}))
            .await;

        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    })
    .await;
}
