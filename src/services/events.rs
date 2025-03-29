use std::sync::Arc;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::{Receiver, Sender}, RwLock};
use tokio_stream::{wrappers::ReceiverStream, Stream};

use crate::{models::Message, random::random_word};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    MessageSent(Message),
}

struct Listener {
    id: String,
    user_id: i64,
    receiver: Sender<BackendEvent>,
}

pub struct ListenerPool {
    listeners: RwLock<Vec<Listener>>,
}

impl ListenerPool {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_listener(&self, user_id: i64, receiver: Sender<BackendEvent>) -> String {
        let mut listeners = self.listeners.write().await;
        let id = random_word(32);
        listeners.push(Listener { id: id.clone(), user_id, receiver });
        id
    }

    pub async fn remove_listener(&self, id: String) {
        let mut listeners = self.listeners.write().await;
        listeners.retain(|listener| listener.id != id);
    }

    pub async fn notify(&self, user_id: i64, event: BackendEvent) {
        let mut need_to_remove = Vec::new();
        {
            let listeners = self.listeners.read().await;
            for listener in listeners.iter() {
                if listener.user_id == user_id {
                    let result = listener.receiver.send(event.clone()).await;
                    if result.is_err() {
                        need_to_remove.push(listener.id.clone());
                    }
                }
            }
        }
        for id in need_to_remove {
            self.remove_listener(id.clone()).await;
            debug!("Listener {} removed", id);
        }
    }
}

pub struct EventService {
    listener_pool: Arc<ListenerPool>,
}

impl EventService {
    pub fn new(listener_pool: Arc<ListenerPool>) -> Self {
        Self { listener_pool }
    }

    pub async fn get_user_stream(&self, user_id: i64) -> ReceiverStream<BackendEvent> {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        self.listener_pool.add_listener(user_id, sender.clone()).await;
        let stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
        stream
    }

    pub async fn notify(&self, user_id: i64, event: BackendEvent) {
        self.listener_pool.notify(user_id, event).await;
    }
}
