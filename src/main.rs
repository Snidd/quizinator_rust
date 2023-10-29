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
    http::{request::Parts, StatusCode},
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
        .route("/", get(get_index).post(using_connection_extractor))
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
async fn get_index(
    State(pool): State<PgPool>,
) -> Result<HelloTemplate<'static>, (StatusCode, String)> {
    /*let res = sqlx::query_scalar("select 'hello world from pg'")
    .fetch_one(&pool)
    .await
    .map_err(internal_error);*/

    Ok(HelloTemplate { name: "world" })
}

// we can extract the connection pool with `State`
async fn get_question(
    Path(user_id): Path<String>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    /*let res = sqlx::query_scalar("select 'hello world from pg'")
    .fetch_one(&pool)
    .await
    .map_err(internal_error);*/

    if let Ok(question) = sqlx::query_as!(Question, "select * from questions limit 1")
        .fetch_optional(&pool)
        .await
    {
        let template = QuestionTemplate {
            text: user_id.clone(),
            answers: vec![String::from("test"), String::from("test2")],
        };

        return HtmlTemplate(template);
    } else {
        return HtmlTemplate::<NotFoundTemplate>(NotFoundTemplate {});
    }
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

struct Question {
    question_id: i32,
    question_text: Option<String>,
    quiz_id: Option<i32>,
}

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

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

async fn using_connection_extractor(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
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
