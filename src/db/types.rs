use std::fmt::Display;

use redis::{from_redis_value, FromRedisValue};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbItem {
    pub id: i64,
    pub user_id: Option<i64>,
    pub message_id: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUser {
    pub id: i64,
    pub email: String,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbMessage {
    pub id: i64,
    pub from_id: i64,
    pub chat_id: i64,
    pub text: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

pub enum Item {
    Message(DbMessage),
    User(DbUser),
}

#[derive(Debug, Clone)]
pub struct DbOTP {
    pub otp: String,
    pub created_at: u64,
}

impl FromRedisValue for DbOTP {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        if let Some((otp, created_at)) = v.split_once(':') {
            Ok(DbOTP {
                otp: otp.to_string(),
                created_at: created_at.parse().unwrap_or(0),
            })
        } else {
            Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Invalid OTP format")))
        }
    }
}

impl Display for DbOTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.otp, self.created_at)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbKnown {
    pub id: i64,
    pub user_id: i64,
    pub item_id: i64,
}
