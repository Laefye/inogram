use std::sync::Arc;

use tokio::sync::{mpsc::Receiver, RwLock};

use crate::{db::{Storage, StorageError}, models::Message};

use super::events::{BackendEvent, EventService};

#[derive(Debug, Clone, Copy)]
pub enum MessageServiceError {
    Storage(StorageError),
    InvalidMessage,
    InvalidChat,
}

impl From<StorageError> for MessageServiceError {
    fn from(storage_error: StorageError) -> Self {
        MessageServiceError::Storage(storage_error)
    }
}

#[derive(Debug, Clone)]
pub struct MessageRequest {
    pub text: String,
}

#[async_trait::async_trait]
pub trait MessageService {
    async fn send_message_to_username(&self, from_id: i64, username: &str, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError>;
    async fn send_message(&self, from_id: i64, chat_id: i64, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError>;
    async fn auto_send_message(&self, from_id: i64, chat_id: Option<i64>, username: Option<&str>, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError>;
}

pub struct ImplMessageService {
    pub storage: Arc<Storage>,
}

impl ImplMessageService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl MessageService for ImplMessageService {
    async fn send_message_to_username(&self, from_id: i64, username: &str, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError> {
        let user = self.storage.get_user_by_username(username).await?;
        if let Some(user) = user {
            let message = self.storage.create_message(from_id, user.id, &message_request.text).await?;
            self.storage.set_known(from_id, user.id).await?;
            self.storage.set_known(user.id, from_id).await?;
            event_service.notify(user.id, BackendEvent::MessageSent(message.clone())).await;
            Ok(message)
        } else {
            Err(MessageServiceError::InvalidChat)
        }
    }

    async fn send_message(&self, from_id: i64, chat_id: i64, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError> {
        if !self.storage.is_known(from_id, chat_id).await? {
            return Err(MessageServiceError::InvalidChat);
        }
        let chat = self.storage.get_user(chat_id).await?;
        if let Some(chat) = chat {
            let message = self.storage.create_message(from_id, chat.id, &message_request.text).await?;
            self.storage.set_known(from_id, chat.id).await?;
            self.storage.set_known(chat.id, from_id).await?;
            event_service.notify(chat.id, BackendEvent::MessageSent(message.clone())).await;
            Ok(message)
        } else {
            Err(MessageServiceError::InvalidChat)
        }
    }

    async fn auto_send_message(&self, from_id: i64, chat_id: Option<i64>, username: Option<&str>, message_request: &MessageRequest, event_service: &EventService) -> Result<Message, MessageServiceError> {
        if let Some(chat_id) = chat_id {
            return self.send_message(from_id, chat_id, message_request, event_service).await;
        } else if let Some(username) = username {
            return self.send_message_to_username(from_id, username, message_request, event_service).await;
        }
        Err(MessageServiceError::InvalidChat)
    }
}
