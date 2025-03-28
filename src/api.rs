use std::sync::Arc;

use axum::{
    extract::State, http::StatusCode, routing::{get, post}, Json, Router
};
use serde::{Deserialize, Serialize};

use crate::{db::Storage, services::user::{self, UserService, UserServiceError}};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

pub async fn run() {
    let app = Router::new()
        .route("/api/v1/users/authenticate", post(try_auth_user))
        .route("/api/v1/users/token", post(get_token))
        .with_state(AppState {
            storage: Arc::new(Storage::new().await),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Serialize)]
pub struct Error {
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthenticateRequest {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthenticateResponse {
    pub is_otp_sent: bool,
}

impl From<UserServiceError> for (StatusCode, Json<Error>) {
    fn from(error: UserServiceError) -> Self {
        match error {
            UserServiceError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, Json(Error { message: "service isn't avaible".to_string() })),
            UserServiceError::InvalidEmail => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid email".to_string() })),
            UserServiceError::InvalidOTP => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid otp".to_string() })),
            UserServiceError::OTPNotSent => panic!("OTP not sent, but it doesn't to send user"),
        }
    }
}

pub async fn try_auth_user(
    State(state): State<AppState>,
    Json(payload): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<Error>)> {
    let user_service = UserService::new(state.storage.clone());
    let is_otp_sent = {
        let result = user_service.authenticate(payload.email).await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => match e {
                UserServiceError::OTPNotSent => Ok(false),
                _ => Err(e),
            },
        }
    }?;
    Ok(Json(AuthenticateResponse { is_otp_sent }))
}

#[derive(Deserialize, Serialize)]
pub struct TokenRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

pub async fn get_token(
    State(state): State<AppState>,
    Json(payload): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<Error>)> {
    let user_service = UserService::new(state.storage.clone());
    let token = user_service.get_token(payload.email, payload.otp).await?;
    Ok(Json(TokenResponse { token }))
}
