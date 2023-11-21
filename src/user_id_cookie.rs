use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{self, request::Parts},
};
use tower_cookies::Cookies;

pub const USER_COOKIE_ID: &str = "user_id";

pub struct UserIdCookie(pub Option<usize>);

#[async_trait]
impl<S> FromRequestParts<S> for UserIdCookie
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(req, state).await?;

        let user_id = cookies
            .get(USER_COOKIE_ID)
            .and_then(|c| c.value().parse::<usize>().ok());

        Ok(UserIdCookie(user_id))
    }
}
