use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::State, http::StatusCode, response::{sse::Event, Sse}, routing::{get, patch, post}, Json, Router
};
use axum_auth::AuthBearer;
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};

use crate::{db::Storage, models::{Message, User}, services::{email::ImplEmailService, events::{BackendEvent, EventService, ListenerPool}, message::{ImplMessageService, MessageRequest, MessageService, MessageServiceError}, user::{ImplUserService, PatchUserField, UserService, UserServiceError}}};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub listener_pool: Arc<ListenerPool>,
}

pub async fn run() {
    let app = Router::new()
        .route("/api/v1/users/authenticate", post(try_auth_user))
        .route("/api/v1/users/token", post(get_token))
        .route("/api/v1/users/", post(create_user))
        .route("/api/v1/users/me", get(get_me))
        .route("/api/v1/users/me", patch(patch_me))
        .route("/api/v1/messages/", post(send_message))
        .route("/api/v1/events/sse", get(get_events))
        .with_state(AppState {
            storage: Arc::new(Storage::new().await),
            listener_pool: Arc::new(ListenerPool::new()),
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
    pub otp_hash: String,
}

impl From<UserServiceError> for (StatusCode, Json<Error>) {
    fn from(service_error: UserServiceError) -> Self {
        log::error!("User Service Error: {:?}", service_error);
        match service_error {
            UserServiceError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Error { message: "service isn't avaible".to_string() })),
            UserServiceError::InvalidEmail => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid email".to_string() })),
            UserServiceError::InvalidOTP => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid otp".to_string() })),
            UserServiceError::OTPNotSent => (StatusCode::BAD_REQUEST, Json(Error { message: "otp not sent".to_string() })),
            UserServiceError::Email(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Error { message: "service isn't avaible".to_string() })),
            UserServiceError::InvalidAuthentication => (StatusCode::UNAUTHORIZED, Json(Error { message: "invalid authentication".to_string() })),
            UserServiceError::UserAlreadyExists => (StatusCode::BAD_REQUEST, Json(Error { message: "user already exists".to_string() })),
            UserServiceError::UsernameUsed => (StatusCode::BAD_REQUEST, Json(Error { message: "username already used".to_string() })),
            UserServiceError::InvalidUsername => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid username".to_string() })),
        }
    }
}

impl From<MessageServiceError> for (StatusCode, Json<Error>) {
    fn from(service_error: MessageServiceError) -> Self {
        log::error!("Message Service Error: {:?}", service_error);
        match service_error {
            MessageServiceError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Error { message: "service isn't avaible".to_string() })),
            MessageServiceError::InvalidMessage => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid message".to_string() })),
            MessageServiceError::InvalidChat => (StatusCode::BAD_REQUEST, Json(Error { message: "invalid chat".to_string() })),
        }
    }
}

pub async fn try_auth_user(
    State(state): State<AppState>,
    Json(payload): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let email_service = ImplEmailService::new();
    let otp_hash = user_service.send_otp(&payload.email, &email_service).await?;
    Ok(Json(AuthenticateResponse { otp_hash }))
}

#[derive(Deserialize, Serialize)]
pub struct TokenRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub has_account: bool,
}

pub async fn get_token(
    State(state): State<AppState>,
    Json(payload): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let token = user_service.verify_otp(&payload.email, &payload.otp).await?;
    let has_account = user_service.authenticate(&token).await?.is_some();
    Ok(Json(TokenResponse { token, has_account }))
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: Option<String>,
}

pub async fn create_user(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let user = user_service.create_user(&token, &payload.first_name, payload.last_name.as_deref()).await?;
    Ok(Json(user))
}

pub async fn get_me(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<User>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let user = user_service.authenticate_with_user(&token).await?;
    Ok(Json(user))
}


#[derive(Deserialize, Serialize)]
pub struct PatchMeRequest {
    pub fields: Vec<PatchUserField>,
}

pub async fn patch_me(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(payload): Json<PatchMeRequest>,
) -> Result<Json<User>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let user = user_service.patch_me(&token, payload.fields).await?;
    Ok(Json(user))
}

#[derive(Deserialize, Serialize)]
pub struct SendMessageRequest {
    pub text: String,
    pub chat_id: Option<i64>,
    pub username: Option<String>,
}

pub async fn send_message(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<Message>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let message_service = ImplMessageService::new(state.storage.clone());
    let event_service = EventService::new(state.listener_pool.clone());
    let user = user_service.authenticate_with_user(&token).await?;
    let message = message_service.auto_send_message(
        user.id,
        payload.chat_id,
        payload.username.as_deref(),
        &MessageRequest { text: payload.text },
        &event_service,
    ).await?;
    Ok(Json(message))
}

pub async fn get_events(
    State(state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<Error>)> {
    let user_service = ImplUserService::new(state.storage.clone());
    let user = user_service.authenticate_with_user(&token).await?;
    let event_service = EventService::new(state.listener_pool.clone());
    let stream = event_service.get_user_stream(user.id).await;
    let sse = Sse::new(stream.map(|event| Ok(Event::default().json_data(event).unwrap())));
    Ok(sse)
}
