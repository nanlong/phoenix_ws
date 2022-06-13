use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;
use nanoid::nanoid;
use serde_json::{json, Map, Value};
use tokio::sync::{broadcast, Mutex};

#[derive(Debug)]
pub struct SocketState {
    pub tx: broadcast::Sender<String>,
}

impl Default for SocketState {
    fn default() -> Self {
        SocketState {
            tx: broadcast::channel(1024).0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Socket {
    id: String,
    joined: bool,
    sender: Option<Arc<Mutex<SplitSink<WebSocket, Message>>>>,
    state: Option<Arc<Mutex<SocketState>>>,
    topic: Option<String>,
    join_ref: Option<String>,
    msg_ref: Option<String>,
    assigns: Map<String, Value>,
    channels: HashSet<String>,
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            id: nanoid!(),
            joined: false,
            sender: None,
            state: None,
            topic: None,
            join_ref: None,
            msg_ref: None,
            assigns: Map::new(),
            channels: HashSet::new(),
        }
    }
}

impl Socket {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn assign(&mut self, key: &str, value: Value) {
        self.assigns.insert(key.to_string(), value);
    }

    pub fn assigns(&self) -> &Map<String, Value> {
        &self.assigns
    }

    pub fn joined(&mut self) {
        self.joined = true;
    }

    pub fn is_joined(&self) -> bool {
        self.joined
    }

    pub fn topic(&self) -> Option<String> {
        self.topic.clone()
    }

    pub fn join_channel(&mut self, topic: &str) {
        self.channels.insert(topic.to_string());
    }

    pub fn leave_channel(&mut self, topic: &str) {
        self.channels.remove(topic);
    }

    pub fn contains_channel(&self, topic: &str) -> bool {
        self.channels.contains(topic)
    }

    pub fn update(
        &mut self,
        sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
        state: Arc<Mutex<SocketState>>,
    ) {
        self.sender = Some(sender);
        self.state = Some(state);
    }

    pub fn from_message(&mut self, message: Value) {
        if message[0].is_null() {
            self.join_ref = None;
        } else {
            self.join_ref = Some(message[0].as_str().unwrap().to_string());
        }

        if message[1].is_null() {
            self.msg_ref = None;
        } else {
            self.msg_ref = Some(message[1].as_str().unwrap().to_string());
        }

        if message[2].is_null() {
            self.topic = None;
        } else {
            self.topic = Some(message[2].as_str().unwrap().to_string());
        }
    }

    pub async fn send(&self, message: Value) {
        if let Some(sender) = &self.sender {
            let mut sender = sender.lock().await;
            if sender
                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                .await
                .is_err()
            {
                tracing::info!("Error sending message");
            }
        }
    }

    pub async fn reply(&self, status: &str, response: Value) {
        let message = if response.is_null() {
            json!({"response": {}, "status": status})
        } else {
            json!({"response": response, "status": status})
        };

        self.push("phx_reply", message).await;
    }

    pub async fn push(&self, event: &str, message: Value) {
        let message = Self::reply_message(
            self.join_ref.clone(),
            self.msg_ref.clone(),
            self.topic.clone(),
            event,
            message,
        );
        self.send(message).await;
    }

    pub async fn boardcast(&self, event: &str, message: Value) {
        self.do_boardcast(None, self.topic(), "boardcast", event, message)
            .await;
    }

    pub async fn boardcast_from(&self, event: &str, message: Value) {
        self.do_boardcast(
            Some(self.id.to_string()),
            self.topic(),
            "boardcast_from",
            event,
            message,
        )
        .await;
    }

    pub async fn do_boardcast(
        &self,
        from: Option<String>,
        topic: Option<String>,
        action: &str,
        event: &str,
        message: Value,
    ) {
        let message =
            Self::reply_message(self.join_ref.clone(), None, topic.clone(), event, message);

        let data = json!({
            "topic": topic.unwrap_or("".to_string()),
            "action": action,
            "from": from,
            "payload": message
        });

        if let Some(state) = &self.state {
            let state = state.lock().await;
            if let Err(_) = state.tx.send(data.to_string()) {
                tracing::error!("Socket {} error sending message", self.id());
            }
        }
    }

    fn reply_message(
        join_ref: Option<String>,
        msg_ref: Option<String>,
        topic: Option<String>,
        event: &str,
        message: Value,
    ) -> Value {
        json!([
            serde_json::to_value(join_ref).unwrap(),
            serde_json::to_value(msg_ref).unwrap(),
            serde_json::to_value(topic).unwrap(),
            serde_json::to_value(event).unwrap(),
            serde_json::to_value(message).unwrap(),
        ])
    }
}
