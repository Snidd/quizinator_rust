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

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use error::{Error, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use templates::{HtmlTemplate, QuestionTemplate};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{net::SocketAddr, time::Duration};
use tower_http::services::ServeDir;

use answer::*;
use question::*;

mod answer;
mod error;
mod question;
mod templates;

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

    let next_question_id = sqlx::query_scalar!(
        "select question_id from questions where question_order > $1 order by question_order limit 1", question.question_order)
        .fetch_optional(&pool)
        .await?;

    let template = QuestionTemplate {
        text: question.question_text,
        answers: answers
            .iter()
            .map(|answer| answer.answer_text.clone())
            .collect(),
        next_question_id: next_question_id,
    };

    return Ok(HtmlTemplate(template));

    /*else {
        return Html(NotFoundTemplate {}.render().unwrap());
        //How to return 404 when question not found?!
    }*/
}
