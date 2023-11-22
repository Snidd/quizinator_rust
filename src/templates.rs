use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::{http::StatusCode, response::Html};

pub struct HtmlTemplate<T>(pub T);

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

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "500.html")]
pub struct InternalServerErrorTemplate {}

#[derive(Template)]
#[template(path = "show_question.html")]
pub struct QuestionTemplate {
    pub text: String,
    pub answers: Vec<String>,
    pub next_question_id: Option<i32>,
}
