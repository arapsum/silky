// use std::sync::Arc;

use axum::http::{HeaderName, HeaderValue};
use axum_extra::extract::cookie::Cookie;
use axum_test::TestServer;
// use serde_json::json;
// use service::AppContext;

#[derive(Debug, Clone)]
pub struct LoggedInUser {
    pub(crate) access_token: HeaderValue,
    pub(crate) refresh_cookie: Cookie<'static>,
    pub(crate) access_cookie: Cookie<'static>,
}

// pub async fn init_login_user(server: &TestServer, _ctx: &Arc<AppContext>) -> LoggedInUser {
//     let email = "john.doe@acme.com";
//     let password = "Password";
//
//     let response = server
//         .post("/auth/sign-in")
//         .json(&json!({
//             "email": email,
//             "password": password
//         }))
//         .await;
//
//     let access_token = response.header("authorization");
//     let refresh_cookie = response.cookies().get("refresh_token").unwrap().to_owned();
//     let access_cookie = response.cookies().get("access_token").unwrap().to_owned();
//
//     LoggedInUser {
//         access_token,
//         refresh_cookie,
//         access_cookie,
//     }
// }

pub async fn login_users(server: &TestServer, body: &serde_json::Value) -> LoggedInUser {
    let response = server
        .post("/auth/login")
        .json(body)
        .do_not_save_cookies()
        .await;

    let access_token = response.header("authorization");
    let refresh_cookie = response.cookies().get("refresh_token").unwrap().to_owned();
    let access_cookie = response.cookies().get("access_token").unwrap().to_owned();

    LoggedInUser {
        access_token,
        refresh_cookie,
        access_cookie,
    }
}

pub fn auth_header(value: HeaderValue) -> (HeaderName, HeaderValue) {
    (HeaderName::from_static("authorization"), value)
}
