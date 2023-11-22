use std::net::SocketAddr;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{ConnectInfo, State};
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::Form;
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};

use crate::error::Result;
use crate::templates::HtmlTemplate;
use crate::user_id_cookie::USER_COOKIE_ID;

#[derive(Template)]
#[template(path = "login_page.html")]
pub struct UserInputTemplate {}

#[derive(Template)]
#[template(path = "login_error.html")]
pub struct LoginErrorForm {
    pub error_msg: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
}

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(pool): State<PgPool>,
    cookies: Cookies,
    Form(login_form): Form<LoginForm>,
) -> Result<impl IntoResponse> {
    println!(
        "got {0} from {1}",
        addr.ip().to_string(),
        login_form.username
    );

    let ip = addr.ip().to_string();
    let existing_user = sqlx::query_as!(
        UserDb,
        "select \"id\", \"unique\", name from users where \"unique\" = $1",
        ip
    )
    .fetch_optional(&pool)
    .await?;

    if let Some(user_db) = existing_user {
        if !user_db.name.eq_ignore_ascii_case(&login_form.username) {
            let login_error_form = LoginErrorForm {
                error_msg: String::from("Användarnamnet matchar inte din IP."),
            };
            return Ok(HtmlTemplate(login_error_form).into_response());
        }
        let mut cookie = Cookie::new(USER_COOKIE_ID, user_db.id.to_string());
        cookie.set_secure(false);
        cookies.add(cookie);
        let mut headers = HeaderMap::new();
        headers.append("HX-Redirect", "/7/question/12".parse().unwrap());
        return Ok((headers, "Should be redirected").into_response());

        //let headers = Headers([(SET_COOKIE, "key=value")]);
        //set cookie here and return it.
    }

    let new_user = sqlx::query_as!(
        UserDb,
        "insert into users
    (\"name\", \"unique\")
  values
    ($1, $2)
  returning \"id\", \"name\", \"unique\"",
        login_form.username,
        ip
    )
    .fetch_one(&pool)
    .await?;

    let mut cookie = Cookie::new(USER_COOKIE_ID, new_user.id.to_string());
    cookie.set_secure(false);
    cookies.add(cookie);
    let mut headers = HeaderMap::new();
    headers.append("HX-Redirect", "/7/question/12".parse().unwrap());
    return Ok((headers, "Should be redirected").into_response());
}

pub async fn get() -> Result<impl IntoResponse> {
    let usertemplate = UserInputTemplate {};

    return Ok(HtmlTemplate(usertemplate).into_response());
}

#[derive(Deserialize)]
pub struct UserDb {
    pub id: i32,
    pub unique: String,
    pub name: String,
}
