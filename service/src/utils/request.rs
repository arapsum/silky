use axum::{
    Extension, Json,
    extract::{FromRequest, FromRequestParts, Path, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use crate::{Error, Report};

pub struct AppJson<T>(pub T);

impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Report;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(Error::JsonRejection)?;

        Ok(Self(value))
    }
}

pub struct AppPath<T>(pub T);

impl<T, S> FromRequestParts<S> for AppPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Report;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(Error::PathRejection)?;

        Ok(Self(value))
    }
}

pub struct AppQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for AppQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Report;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(Error::QueryRejection)?;

        Ok(Self(value))
    }
}

pub struct AppExtension<T>(pub T);

impl<T, S> FromRequestParts<S> for AppExtension<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = Report;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(value) = Extension::<T>::from_request_parts(parts, state)
            .await
            .map_err(Error::ExtensionRejection)?;

        Ok(Self(value))
    }
}
