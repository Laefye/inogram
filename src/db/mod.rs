use std::time::SystemTime;

use redis::AsyncCommands;
use types::{DbItem, DbOTP};

use crate::models::{self, User};

mod types;

pub struct Storage {
    pool: sqlx::Pool<sqlx::Postgres>,
    redis: redis::Client,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageError {
    Internal,
}

impl From<sqlx::Error> for StorageError {
    fn from(_: sqlx::Error) -> Self {
        StorageError::Internal
    }
}

impl From<redis::RedisError> for StorageError {
    fn from(_: redis::RedisError) -> Self {
        StorageError::Internal
    }
}

impl Storage {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

        let pool = sqlx::PgPool::connect(&database_url).await.expect("Failed to connect to the database");
        let redis = redis::Client::open(redis_url).expect("Failed to connect to Redis");

        Storage { pool, redis }
    }

    async fn create_item(&self, item: &types::Item) -> Result<DbItem, StorageError> {
        match item {
            types::Item::Message(db_message) => {
                let query = sqlx::query_as!(
                        types::DbItem,
                        r#"
                        INSERT INTO public.items (message_id)
                            VALUES ($1)
                            RETURNING *
                        "#,
                        db_message.id,
                    )
                    .fetch_one(&self.pool)
                    .await?;
                Ok(query.into())
            },
            types::Item::User(db_user) => {
                let query = sqlx::query_as!(
                        types::DbItem,
                        r#"
                        INSERT INTO public.items (user_id)
                            VALUES ($1)
                            RETURNING *
                        "#,
                        db_user.id,
                    )
                    .fetch_one(&self.pool)
                    .await?;
                Ok(query.into())
            },
        }
    }  

    pub async fn create_user(&self, email: &str, username: Option<&str>, first_name: &str, last_name: Option<&str>) -> Result<models::User, StorageError> {
        let query = sqlx::query_as!(
                types::DbUser,
                r#"
                INSERT INTO public.users (email, username, first_name, last_name)
                    VALUES ($1, $2, $3, $4)
                    RETURNING *
                "#,
                email,
                username,
                first_name,
                last_name
            )
            .fetch_one(&self.pool)
            .await?;
        let item = self.create_item(&types::Item::User(query.clone())).await?;
        let user = models::User {
            id: item.id,
            email: query.email,
            username: query.username,
            first_name: query.first_name,
            last_name: query.last_name,
            created_at: query.created_at.and_utc().timestamp() as usize,
        };
        Ok(user)
    }

    pub async fn create_message(&self, from_id: i64, chat_id: i64, text: &str) -> Result<models::Message, StorageError> {
        let db_message = sqlx::query_as!(
                types::DbMessage,
                r#"
                INSERT INTO public.messages (from_id, chat_id, text)
                    VALUES ($1, $2, $3)
                    RETURNING *
                "#,
                from_id,
                chat_id,
                text
            )
            .fetch_one(&self.pool)
            .await?;
        let item = self.create_item(&types::Item::Message(db_message.clone())).await?;
        let message = models::Message {
            id: item.id,
            from_id: db_message.from_id,
            chat_id: db_message.chat_id,
            text: db_message.text,
            created_at: db_message.created_at.and_utc().timestamp() as usize,
        };
        Ok(message)
    }

    pub async fn get_user(&self, item_id: i64) -> Result<Option<models::User>, StorageError> {
        let db_user = self.get_db_user(item_id).await?;
        if db_user.is_none() {
            return Ok(None);
        }
        let db_user = db_user.unwrap();
        let user = models::User {
            id: item_id,
            email: db_user.email,
            username: db_user.username,
            first_name: db_user.first_name,
            last_name: db_user.last_name,
            created_at: db_user.created_at.and_utc().timestamp() as usize,
        };
        Ok(Some(user))
    }

    async fn get_item_of_user(&self, user_id: i64) -> Result<Option<types::DbItem>, StorageError> {
        let query = sqlx::query_as!(
                types::DbItem,
                r#"
                SELECT public.items.* 
                    FROM public.items
                    JOIN public.users ON items.user_id = users.id
                    WHERE users.id = $1
                "#,
                user_id
            )
            .fetch_optional(&self.pool)
            .await?;
        Ok(query)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<models::User>, StorageError> {
        let query = sqlx::query_as!(
                types::DbUser,
                r#"
                SELECT public.users.* 
                    FROM public.users
                    WHERE users.email = $1
                "#,
                email
            )
            .fetch_optional(&self.pool)
            .await?;
        if let Some(db_user) = query {
            let item = self.get_item_of_user(db_user.id).await?;
            if let Some(item) = item {
                let user = models::User {
                    id: item.id,
                    email: db_user.email,
                    username: db_user.username,
                    first_name: db_user.first_name,
                    last_name: db_user.last_name,
                    created_at: db_user.created_at.and_utc().timestamp() as usize,
                };
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    pub async fn get_message(&self, item_id: i64) -> Result<Option<models::Message>, StorageError> {
        let query = sqlx::query_as!(
                types::DbMessage,
                r#"
                SELECT public.messages.* 
                    FROM public.items
                    JOIN public.messages ON items.message_id = messages.id
                    WHERE items.id = $1
                "#,
                item_id
            )
            .fetch_optional(&self.pool)
            .await?;
        Ok(query.map(|db_message| {
            models::Message {
                id: item_id,
                from_id: db_message.from_id,
                chat_id: db_message.chat_id,
                text: db_message.text,
                created_at: db_message.created_at.and_utc().timestamp() as usize,
            }
        }))
    }

    pub async fn store_otp(&self, email: &str, otp: &str) -> Result<(), StorageError> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        con.set_ex::<_, _, ()>(
            format!("otp:{}", email),
            DbOTP {
                otp: otp.to_string(),
                created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            }.to_string(),
            300).await?;
        Ok(())
    }

    pub async fn get_otp(&self, email: &str) -> Result<Option<(String, SystemTime)>, StorageError> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let key = format!("otp:{}", email);
        let db_otp: Option<DbOTP> = con.get(&key).await?;
        if let Some(db_otp) = db_otp {
            let created_at = SystemTime::UNIX_EPOCH + std::time::Duration::new(db_otp.created_at, 0);
            Ok(Some((db_otp.otp, created_at)))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_otp(&self, email: &str) -> Result<(), StorageError> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;
        let key = format!("otp:{}", email);
        let _:() = con.del(key).await?;
        Ok(())
    }

    async fn get_db_user(&self, item_id: i64) -> Result<Option<types::DbUser>, StorageError> {
        let query = sqlx::query_as!(
                types::DbUser,
                r#"
                SELECT public.users.* 
                    FROM public.items
                    JOIN public.users ON items.user_id = users.id
                    WHERE items.id = $1
                "#,
                item_id
            )
            .fetch_optional(&self.pool)
            .await?;
        Ok(query)
    }

    pub async fn update_user(&self, user: &models::User) -> Result<models::User, StorageError> {
        let db_user = self.get_db_user(user.id).await?;
        if db_user.is_none() {
            return Err(StorageError::Internal);
        }
        let db_user = db_user.unwrap();
        let query = sqlx::query_as!(
                types::DbUser,
                r#"
                UPDATE public.users
                    SET first_name = $1, last_name = $2, username = $3
                    WHERE id = $4
                    RETURNING *
                "#,
                user.first_name,
                user.last_name,
                user.username,
                db_user.id
            )
            .fetch_one(&self.pool)
            .await?;
        let item = self.get_item_of_user(query.id).await?;
        if let Some(item) = item {
            let updated_user = models::User {
                id: item.id,
                email: query.email,
                username: query.username,
                first_name: query.first_name,
                last_name: query.last_name,
                created_at: query.created_at.and_utc().timestamp() as usize,
            };
            return Ok(updated_user)
        }
        Err(StorageError::Internal)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<models::User>, StorageError> {
        let query = sqlx::query_as!(
                types::DbUser,
                r#"
                SELECT public.users.* 
                    FROM public.users
                    WHERE LOWER(users.username) = LOWER($1)
                "#,
                username
            )
            .fetch_optional(&self.pool)
            .await?;
        if query.is_none() {
            return Ok(None);
        }
        let db_user = query.unwrap();
        let item = self.get_item_of_user(db_user.id).await?;
        if item.is_none() {
            return Ok(None);
        }
        let item = item.unwrap();
        let user = models::User {
            id: item.id,
            email: db_user.email,
            username: db_user.username,
            first_name: db_user.first_name,
            last_name: db_user.last_name,
            created_at: db_user.created_at.and_utc().timestamp() as usize,
        };
        Ok(Some(user))
    }

    pub async fn is_known(&self, user_id: i64, item_id: i64) -> Result<bool, StorageError> {
        let query = sqlx::query!(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM public.known
                    WHERE user_id = $1 AND item_id = $2
                )
                "#,
                user_id,
                item_id
            )
            .fetch_one(&self.pool)
            .await?;
        Ok(query.exists.unwrap_or(false))
    }

    pub async fn set_known(&self, user_id: i64, item_id: i64) -> Result<(), StorageError> {
        if self.is_known(user_id, item_id).await? {
            return Ok(());
        }
        sqlx::query!(
                r#"
                INSERT INTO public.known (user_id, item_id)
                    VALUES ($1, $2)
                "#,
                user_id,
                item_id
            )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

