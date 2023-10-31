use askama_axum::{IntoResponse, Response, Template};
use axum::{http::StatusCode, response::Html};

use crate::templates::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request path not found")]
    NotFound,

    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    /*fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }*/
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self {
            Error::NotFound => (
                StatusCode::NOT_FOUND,
                Html(NotFoundTemplate {}.render().unwrap()),
            ),
            Error::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(InternalServerErrorTemplate {}.render().unwrap()),
            ),
            Error::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(InternalServerErrorTemplate {}.render().unwrap()),
            ),
        };

        return body.into_response();
    }
}
