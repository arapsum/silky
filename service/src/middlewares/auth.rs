use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    RequestPartsExt,
    body::Body,
    http::{Request, Response},
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Cookie, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};

use crate::{AppContext, Error};

#[derive(Clone)]
pub struct AuthLayer {
    state: Arc<AppContext>,
}

impl AuthLayer {
    #[must_use]
    pub const fn new(state: Arc<AppContext>) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    state: Arc<AppContext>,
}

impl<S> AuthService<S> {
    #[must_use]
    pub const fn new(inner: S, state: Arc<AppContext>) -> Self {
        Self { inner, state }
    }
}

impl<S, B> Service<Request<B>> for AuthService<S>
where
    S: Service<Request<B>, Response = Response<Body>, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let state = self.state.clone();
        let clone = self.inner.clone();

        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();

            let access_token = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
                Ok(header) => Some(header.token().to_string()),
                Err(e) => {
                    // if access-token not in Authorisation Header, check Cookies.

                    if matches!(e.reason(), TypedHeaderRejectionReason::Missing) {
                        parts.extract::<TypedHeader<Cookie>>().await.ok().and_then(
                            |TypedHeader(cookie)| {
                                cookie.get("access_token").map(ToString::to_string)
                            },
                        )
                    } else {
                        return Ok::<Response<Body>, Self::Error>(Error::InvalidToken.response());
                    }
                }
            };

            let Some(token) = access_token else {
                return Ok(Error::MissingCredentials.response());
            };

            let token_claims = match state.auth().verify_access_token(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    return Ok(e.into_response());
                }
            };

            let mut req = Request::from_parts(parts, body);
            req.extensions_mut().insert(token_claims);

            inner.call(req).await
        })
    }
}
