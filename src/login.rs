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
#[template(path = "user_input.html")]
pub struct UserInputTemplate {}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
}

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(login_form): Form<LoginForm>,
) -> Result<impl IntoResponse> {
    println!(
        "got {0} from {1}",
        addr.ip().to_string(),
        login_form.username
    );
    return Err(crate::error::Error::NotFound);
    let mut headers = HeaderMap::new();
    headers.append("HX-Redirect", "/7/question/12".parse().unwrap());
    //HX-Redirect: /question/xx
    return Ok((headers, "Should be redirected"));
}

pub async fn get() -> Result<impl IntoResponse> {
    let usertemplate = UserInputTemplate {};

    return Ok(HtmlTemplate(usertemplate).into_response());
}
