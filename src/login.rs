use std::net::SocketAddr;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::Form;
use axum_macros::debug_handler;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::Result;
use crate::templates::HtmlTemplate;

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
    Form(login_form): Form<LoginForm>,
) -> Result<impl IntoResponse> {
    println!(
        "got {0} from {1}",
        addr.ip().to_string(),
        login_form.username
    );

    let login_error_form = LoginErrorForm {
        error_msg: String::from("Testing"),
    };

    /*SELECT * FROM users
    WHERE name = ${username.toString()}
    ORDER BY id ASC LIMIT 100 */
    return Ok(HtmlTemplate(login_error_form).into_response());
    //return Err(crate::error::Error::NotFound);
    let mut headers = HeaderMap::new();
    headers.append("HX-Redirect", "/7/question/12".parse().unwrap());
    //HX-Redirect: /question/xx
    return Ok((headers, "Should be redirected").into_response());
}

pub async fn get() -> Result<impl IntoResponse> {
    let usertemplate = UserInputTemplate {};

    return Ok(HtmlTemplate(usertemplate).into_response());
}
