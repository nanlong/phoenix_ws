use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;

use super::socket::Socket;

pub struct Endpoint();

impl Endpoint {
    pub async fn boardcast(socket: Arc<Mutex<Socket>>, topic: &str, event: &str, message: Value) {
        let socket = socket.lock().await;

        socket
            .do_boardcast(None, Some(topic.to_string()), "boardcast", event, message)
            .await;
    }

    pub async fn boardcast_from(
        socket: Arc<Mutex<Socket>>,
        topic: &str,
        event: &str,
        message: Value,
    ) {
        let socket = socket.lock().await;

        socket
            .do_boardcast(
                Some(socket.id().to_string()),
                Some(topic.to_string()),
                "boardcast_from",
                event,
                message,
            )
            .await;
    }
}
