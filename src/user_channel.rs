use std::{collections::HashMap, sync::Arc};

use futures::Future;
use serde_json::Value;
use tokio::sync::Mutex;

use super::async_callback::AsyncCallback;
use super::socket::Socket;

pub struct UserChannel {
    topic: String,
    handles: HashMap<String, Box<dyn AsyncCallback + Send + Sync>>,
}

impl UserChannel {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            handles: HashMap::new(),
        }
    }

    pub fn topic(&self) -> String {
        self.topic.to_string()
    }

    pub fn handle_event<F, R>(&mut self, event: &str, callback: F)
    where
        F: Fn(Value, Arc<Mutex<Socket>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.handles.insert(event.to_string(), Box::new(callback));
    }

    pub fn join<F, R>(&mut self, callback: F)
    where
        F: Fn(Value, Arc<Mutex<Socket>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.handle_event("phx_join", callback);
    }

    pub fn leave<F, R>(&mut self, callback: F)
    where
        F: Fn(Value, Arc<Mutex<Socket>>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.handle_event("phx_leave", callback);
    }

    pub async fn dispatch(&mut self, event: &str, message: Value, socket: Arc<Mutex<Socket>>) {
        if let Some(callback) = self.handles.get(event) {
            callback.call(message[4].clone(), socket.clone()).await;
        };

        match event {
            "phx_join" => socket
                .lock()
                .await
                .join_channel(message[2].as_str().unwrap()),
            "phx_leave" => socket
                .lock()
                .await
                .leave_channel(message[2].as_str().unwrap()),
            _ => (),
        }
    }
}
