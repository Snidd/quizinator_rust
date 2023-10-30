//! Example of application using <https://github.com/launchbadge/sqlx>
//!
//! Run with
//!
//! ```not_rust
//! cargo run -p example-sqlx-postgres
//! ```
//!
//! Test with curl:
//!
//! ```not_rust
//! curl 127.0.0.1:3000
//! curl -X POST 127.0.0.1:3000
//! ```

use askama::Template;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{header, request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use maud::{html, Markup, DOCTYPE};
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    // build our application with some routes
    let app = Router::new()
        //.route("/", get(get_index).post(using_connection_extractor))
        .route("/:question_id", get(get_question))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// we can extract the connection pool with `State`
/*async fn get_index(
    State(pool): State<PgPool>,
) -> Result<HelloTemplate<'static>, (StatusCode, String)> {
    /*let res = sqlx::query_scalar("select 'hello world from pg'")
    .fetch_one(&pool)
    .await
    .map_err(internal_error);*/

    Ok(HelloTemplate { name: "world" })
}*/

struct Question {
    pub question_id: i32,
    pub question_text: String,
    pub quiz_id: i32,
}

struct Answer {
    pub answer_id: i32,
    pub answer_text: String,
    pub answer_is_correct: bool,
    pub question_id: i32,
}

// we can extract the connection pool with `State`

async fn get_question(
    Path(question_id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse> {
    let question = sqlx::query_as!(
        Question,
        "select * from questions where question_id = $1",
        question_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(Error::NotFound)?;

    let answers = sqlx::query_as!(
        Answer,
        "select * from answers where question_id = $1",
        question.question_id
    )
    .fetch_all(&pool)
    .await?;

    let template = QuestionTemplate {
        text: question.question_text,
        answers: answers
            .iter()
            .map(|answer| answer.answer_text.clone())
            .collect(),
    };

    return Ok(Html(template.render().unwrap()));

    /*else {
        return Html(NotFoundTemplate {}.render().unwrap());
        //How to return 404 when question not found?!
    }*/
}

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

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "500.html")]
struct InternalServerErrorTemplate {}

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "show_question.html")]
struct QuestionTemplate {
    pub(crate) text: String,
    pub(crate) answers: Vec<String>,
}

async fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
