use std::{pin::Pin, sync::Arc, time::{Duration, SystemTime}};

use jsonwebtoken::EncodingKey;
use serde::{Deserialize, Serialize};

use crate::{db::{Storage, StorageError}, models::User, random};

use super::email::{EmailService, EmailServiceError};

#[derive(Debug, Clone, Copy)]
pub enum UserServiceError {
    Storage(StorageError),
    Email(EmailServiceError),
    InvalidEmail,
    InvalidOTP,
    OTPNotSent,
    InvalidAuthentication,
    UserAlreadyExists,
    UsernameUsed,
    InvalidUsername,
}

impl From<StorageError> for UserServiceError {
    fn from(storage_error: StorageError) -> Self {
        UserServiceError::Storage(storage_error)
    }
}

impl From<EmailServiceError> for UserServiceError {
    fn from(email_error: EmailServiceError) -> Self {
        UserServiceError::Email(email_error)
    }
}

#[async_trait::async_trait]
pub trait UserService {
    async fn send_otp<T: EmailService + Send + Sync>(&self, email: &str, email_service: &T) -> Result<String, UserServiceError>;
    async fn verify_otp(&self, email: &str, otp: &str) -> Result<String, UserServiceError>;
    async fn authenticate(&self, token: &str) -> Result<Option<User>, UserServiceError>;
    async fn authenticate_with_user(&self, token: &str) -> Result<User, UserServiceError> {
        if let Some(user) = self.authenticate(token).await? {
            return Ok(user);
        }
        Err(UserServiceError::InvalidAuthentication)
    }
    async fn create_user(&self, token: &str, first_name: &str, last_name: Option<&str>) -> Result<User, UserServiceError>;
    async fn patch_me(&self, token: &str, fields: Vec<PatchUserField>) -> Result<User, UserServiceError>;
}

pub struct ImplUserService {
    storage: Arc<Storage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchUserField {
    pub name: String,
    pub value: Option<String>,
}

impl ImplUserService {
    pub fn new(storage: Arc<Storage>) -> Self {
        ImplUserService { storage }
    }

    fn create_jwt_token(&self, email: &str) -> String {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let claims = JwtClaims {
            sub: email.to_string(),
            exp: SystemTime::now()
                .checked_add(Duration::from_secs(3600)).unwrap()
                .duration_since(SystemTime::UNIX_EPOCH).unwrap()
                .as_secs() as usize,
        };
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();
        token
    }

    fn verify_jwt_token(&self, token: &str) -> Option<String> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token_data = jsonwebtoken::decode::<JwtClaims>(token, &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()), &jsonwebtoken::Validation::default()).ok()?;
        Some(token_data.claims.sub)
    }

    fn check_username(&self, username: &str) -> Result<(), UserServiceError> {
        if username.is_empty() {
            return Err(UserServiceError::InvalidUsername);
        }
        if username.len() < 3 || username.len() > 20 {
            return Err(UserServiceError::InvalidUsername);
        }
        if username.chars().filter(|c| *c != '_').any(|c| !c.is_alphanumeric()) {
            return Err(UserServiceError::InvalidUsername);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl UserService for ImplUserService {
    async fn send_otp<T: EmailService + Send + Sync>(&self, email: &str, email_service: &T) -> Result<String, UserServiceError> {
        if email.is_empty() || !email.contains('@') {
            return Err(UserServiceError::InvalidEmail);
        }
        let stored_otp = self.storage.get_otp(email).await?;
        if stored_otp.is_some() {
            return Err(UserServiceError::OTPNotSent);
        }
        let otp = random::generate_otp();
        self.storage.store_otp(email, &otp).await?;
        email_service.send_otp(&email, &otp).await?;
        let otp_hash = random::random_hash(&otp);
        Ok(otp_hash)
    }

    async fn verify_otp(&self, email: &str, otp: &str) -> Result<String, UserServiceError> {
        if email.is_empty() || !email.contains('@') {
            return Err(UserServiceError::InvalidEmail);
        }
        let stored_otp = self.storage.get_otp(email).await?;
        if stored_otp.is_none() {
            return Err(UserServiceError::InvalidOTP);
        }
        let (stored_otp, _) = stored_otp.unwrap();
        if stored_otp != otp {
            return Err(UserServiceError::InvalidOTP);
        }
        self.storage.delete_otp(email).await?;
        Ok(self.create_jwt_token(email))
    }

    async fn authenticate(&self, token: &str) -> Result<Option<User>, UserServiceError> {
        if let Some(email) = self.verify_jwt_token(token) {
            let user = self.storage.get_user_by_email(&email).await?;
            return Ok(user);
        }
        Err(UserServiceError::InvalidAuthentication)
    }

    async fn create_user(&self, token: &str, first_name: &str, last_name: Option<&str>) -> Result<User, UserServiceError> {
        let email = self.verify_jwt_token(token).ok_or(UserServiceError::InvalidAuthentication)?;
        let user = self.storage.get_user_by_email(&email).await?;
        if user.is_some() {
            return Err(UserServiceError::UserAlreadyExists);
        }
        let user = self.storage.create_user(&email, None, first_name, last_name).await?;
        Ok(user)
    }

    async fn patch_me(&self, token: &str, fields: Vec<PatchUserField>) -> Result<User, UserServiceError> {
        let mut user = self.authenticate_with_user(token).await?;
        for field in fields {
            match field.name.as_str() {
                "first_name" => user.first_name = field.value.unwrap_or_default(),
                "last_name" => user.last_name = field.value,
                "username" => user.username = field.value,
                _ => (),
            }
        }
        if let Some(username) = &user.username {
            self.check_username(username)?;
            let existing_user = self.storage.get_user_by_username(username).await?;
            if existing_user.is_some() && existing_user.unwrap().id != user.id {
                return Err(UserServiceError::UsernameUsed);
            }
        }
        self.storage.update_user(&user).await?;
        Ok(user)
    }
}
