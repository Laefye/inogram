use log::debug;

#[derive(Debug, Clone, Copy)]
pub enum EmailServiceError {
    // Internal(String),
}

#[async_trait::async_trait]
pub trait EmailService {
    async fn send_otp(&self, email: &str, otp: &str) -> Result<(), EmailServiceError>;
}

pub struct ImplEmailService {
}

impl ImplEmailService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl EmailService for ImplEmailService {
    async fn send_otp(&self, email: &str, otp: &str) -> Result<(), EmailServiceError> {
        debug!("Sending OTP to email: {} - {}", email, otp);
        Ok(())
    }
}
