use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::{Method, Request, Response},
};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::{
    AppContext, Error, access_control::PermissionName, context::Claims, models::Permission,
};

#[derive(Clone)]
pub struct RbacLayer {
    state: Arc<AppContext>,
    required_permission: Option<PermissionName>,
    route_permissions: Vec<(Method, Option<&'static str>, PermissionName)>,
    allow_customers: bool,
    deny_customers: bool,
}

impl RbacLayer {
    #[must_use]
    pub const fn new(state: Arc<AppContext>, required_permission: PermissionName) -> Self {
        Self {
            state,
            required_permission: Some(required_permission),
            route_permissions: Vec::new(),
            allow_customers: false,
            deny_customers: false,
        }
    }

    /// Creates an RBAC layer whose required permission depends on the request method.
    ///
    /// This is useful when multiple operations share a path, such as reading a
    /// collection with `GET` and creating a record with `POST`.
    #[must_use]
    pub fn for_methods(
        state: Arc<AppContext>,
        method_permissions: impl IntoIterator<Item = (Method, PermissionName)>,
    ) -> Self {
        Self {
            state,
            required_permission: None,
            route_permissions: method_permissions
                .into_iter()
                .map(|(method, permission)| (method, None, permission))
                .collect(),
            allow_customers: false,
            deny_customers: false,
        }
    }

    /// Creates an RBAC layer with method and optional path-specific rules.
    ///
    /// Rules are evaluated in declaration order. Place path-specific rules
    /// before a method-wide fallback rule for the same HTTP method.
    #[must_use]
    pub fn for_routes(
        state: Arc<AppContext>,
        route_permissions: impl IntoIterator<Item = (Method, Option<&'static str>, PermissionName)>,
    ) -> Self {
        Self {
            state,
            required_permission: None,
            route_permissions: route_permissions.into_iter().collect(),
            allow_customers: false,
            deny_customers: false,
        }
    }

    #[must_use]
    pub const fn allow_customers(mut self) -> Self {
        self.allow_customers = true;
        self
    }

    #[must_use]
    pub const fn deny_customers(mut self) -> Self {
        self.deny_customers = true;
        self
    }
}

impl<S> Layer<S> for RbacLayer {
    type Service = RbacService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            state: self.state.clone(),
            required_permission: self.required_permission,
            route_permissions: self.route_permissions.clone(),
            allow_customers: self.allow_customers,
            deny_customers: self.deny_customers,
        }
    }
}

#[derive(Clone)]
pub struct RbacService<S> {
    inner: S,
    state: Arc<AppContext>,
    required_permission: Option<PermissionName>,
    route_permissions: Vec<(Method, Option<&'static str>, PermissionName)>,
    allow_customers: bool,
    deny_customers: bool,
}

impl<S> RbacService<S> {
    #[must_use]
    pub const fn new(
        inner: S,
        state: Arc<AppContext>,
        required_permission: PermissionName,
    ) -> Self {
        Self {
            inner,
            state,
            required_permission: Some(required_permission),
            route_permissions: Vec::new(),
            allow_customers: false,
            deny_customers: false,
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
        let required_permission = self.required_permission;
        let route_permissions = self.route_permissions.clone();
        let allow_customers = self.allow_customers;
        let deny_customers = self.deny_customers;
        let clone = self.inner.clone();

        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let required_permission = required_permission.or_else(|| {
                route_permissions
                    .iter()
                    .find(|(method, path_suffix, _)| {
                        method == req.method()
                            && path_suffix.is_none_or(|suffix| req.uri().path().ends_with(suffix))
                    })
                    .map(|(_, _, permission)| *permission)
            });
            let Some(required_permission) = required_permission else {
                return Ok(Error::Forbidden.response());
            };

            let Some(claims) = req.extensions().get::<Claims>() else {
                return Ok(Error::MissingCredentials.response());
            };

            let Ok(user_pid) = Uuid::parse_str(claims.sub()) else {
                return Ok(Error::Forbidden.response());
            };

            if crate::models::User::has_role(state.db(), user_pid, "customer")
                .await
                .unwrap_or(false)
            {
                if deny_customers {
                    return Ok(Error::Forbidden.response());
                }
                if allow_customers {
                    return inner.call(req).await;
                }
            }

            let granted = Permission::is_granted_to_user_role(
                state.db(),
                user_pid,
                required_permission.as_str(),
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
