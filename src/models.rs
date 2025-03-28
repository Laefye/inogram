#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub from_id: i64,
    pub chat_id: i64,
    pub text: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}
