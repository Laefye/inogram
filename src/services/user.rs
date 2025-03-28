use std::{sync::Arc, time::SystemTime};

use log::debug;

use crate::{db::{Storage, StorageError}, random};

pub struct UserService {
    storage: Arc<Storage>,
}

pub enum UserServiceError {
    Internal,
    InvalidEmail,
    InvalidOTP,
    OTPNotSent,
}

impl From<StorageError> for UserServiceError {
    fn from(_: StorageError) -> Self {
        UserServiceError::Internal
    }
}

impl UserService {
    pub fn new(storage: Arc<Storage>) -> Self {
        UserService { storage }
    }

    pub async fn authenticate(&self, email: String) -> Result<(), UserServiceError> {
        if email.is_empty() || !email.contains('@') {
            return Err(UserServiceError::InvalidEmail);
        }
        let stored_otp = self.storage.get_otp(&email).await?;
        if stored_otp.is_some() {
            return Err(UserServiceError::OTPNotSent);
        }
        let otp = random::generate_otp();
        self.storage.store_otp(&email, &otp).await?;
        debug!("Generated OTP for {}: {}", email, otp);
        Ok(())
    }

    pub async fn get_token(&self, email: String, otp: String) -> Result<String, UserServiceError> {
        if email.is_empty() || !email.contains('@') {
            return Err(UserServiceError::InvalidEmail);
        }
        let stored_otp = self.storage.get_otp(&email).await?;
        if stored_otp.is_none() {
            return Err(UserServiceError::InvalidOTP);
        }
        let (stored_otp, _) = stored_otp.unwrap();
        if stored_otp != otp {
            return Err(UserServiceError::InvalidOTP);
        }
        self.storage.delete_otp(&email).await?;
        Ok("SOME_TOKEN".to_string())
    }
}
