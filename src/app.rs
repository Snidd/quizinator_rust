use std::time::Duration;

use crate::error::{Error, Result};
use crate::question::Question;
use crate::templates::QuestionTemplate;

use crate::{
    answer::*,
    templates::{HtmlTemplate, UserInputTemplate},
    user_id_cookie::{UserIdCookie, USER_COOKIE_ID},
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct App {}

impl App {
    pub async fn new() -> Router {
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
            .route("/", get(get_user))
            .nest_service("/static", ServeDir::new("static"))
            .with_state(pool)
            .layer(CookieManagerLayer::new());

        // run it with hyper
        return app;
    }
}

async fn get_user(
    State(_pool): State<PgPool>,
    cookies: Cookies,
    user_id: UserIdCookie,
) -> Result<impl IntoResponse> {
    cookies.add(Cookie::new(USER_COOKIE_ID, "123"));

    let usertemplate = UserInputTemplate {};

    if user_id.0.is_some() {
        return Ok(Redirect::to("/5").into_response());
    }

    return Ok(HtmlTemplate(usertemplate).into_response());
}

async fn get_question(
    Path(question_id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse> {
    let question = sqlx::query_as!(
        Question,
        "select * from questions where id = $1",
        question_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(Error::NotFound)?;

    let answers = sqlx::query_as!(
        Answer,
        "select * from answers where question_id = $1",
        question.id
    )
    .fetch_all(&pool)
    .await?;

    let next_question_id = sqlx::query_scalar!(
        "select id from questions where \"order\" > $1 order by \"order\" limit 1",
        question.order
    )
    .fetch_optional(&pool)
    .await?;

    let template = QuestionTemplate {
        text: question.text,
        answers: answers.iter().map(|answer| answer.text.clone()).collect(),
        next_question_id: next_question_id,
    };

    return Ok(HtmlTemplate(template));

    /*else {
        return Html(NotFoundTemplate {}.render().unwrap());
        //How to return 404 when question not found?!
    }*/
}
