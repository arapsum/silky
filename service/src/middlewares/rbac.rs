use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::{Request, Response},
};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::{AppContext, Error, context::Claims, models::Permission};

#[derive(Clone)]
pub struct RbacLayer {
    state: Arc<AppContext>,
    allowed_roles: Vec<String>,
    required_permission: String,
}

impl RbacLayer {
    #[must_use]
    pub fn new<I, R, P>(state: Arc<AppContext>, allowed_roles: I, required_permission: P) -> Self
    where
        I: IntoIterator<Item = R>,
        R: AsRef<str>,
        P: AsRef<str>,
    {
        let allowed_roles = allowed_roles
            .into_iter()
            .map(|role| role.as_ref().trim().to_lowercase())
            .filter(|role| !role.is_empty())
            .collect();
        let required_permission = required_permission.as_ref().trim().to_lowercase();

        Self {
            state,
            allowed_roles,
            required_permission,
        }
    }
}

impl<S> Layer<S> for RbacLayer {
    type Service = RbacService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            state: self.state.clone(),
            allowed_roles: self.allowed_roles.clone(),
            required_permission: self.required_permission.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RbacService<S> {
    inner: S,
    state: Arc<AppContext>,
    allowed_roles: Vec<String>,
    required_permission: String,
}

impl<S> RbacService<S> {
    #[must_use]
    pub const fn new(
        inner: S,
        state: Arc<AppContext>,
        allowed_roles: Vec<String>,
        required_permission: String,
    ) -> Self {
        Self {
            inner,
            state,
            allowed_roles,
            required_permission,
        }
    }
}

impl<S, B> Service<Request<B>> for RbacService<S>
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
        let allowed_roles = self.allowed_roles.clone();
        let required_permission = self.required_permission.clone();
        let clone = self.inner.clone();

        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let Some(claims) = req.extensions().get::<Claims>() else {
                return Ok(Error::MissingCredentials.response());
            };

            let Ok(user_pid) = Uuid::parse_str(claims.sub()) else {
                return Ok(Error::Forbidden.response());
            };

            let granted = Permission::is_granted_to_user_role(
                state.db(),
                user_pid,
                &allowed_roles,
                &required_permission,
            )
            .await;

            match granted {
                Ok(true) => inner.call(req).await,
                Ok(false) => Ok(Error::Forbidden.response()),
                Err(err) => Ok(Error::Model(err).response()),
            }
        })
    }
}
