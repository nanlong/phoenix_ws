use std::sync::Arc;

use futures::{future::BoxFuture, Future};
use serde_json::Value;
use tokio::sync::Mutex;

use super::socket::Socket;

pub trait AsyncCallback {
    fn call(&self, payload: Value, socket: Arc<Mutex<Socket>>) -> BoxFuture<'static, ()>;
}

impl<T, F> AsyncCallback for T
where
    T: Fn(Value, Arc<Mutex<Socket>>) -> F,
    F: Future<Output = ()> + Send + 'static,
{
    fn call(&self, payload: Value, socket: Arc<Mutex<Socket>>) -> BoxFuture<'static, ()> {
        Box::pin(self(payload, socket))
    }
}
