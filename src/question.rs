pub struct Question {
    pub id: i32,
    pub text: String,
    pub order: i32,
    pub quiz_id: i32,
    pub ingress: Option<String>,
}
