use async_trait::async_trait;

pub mod manager;
pub use manager::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "conn_mgr_ws")] {
        pub mod websocket;
    }
}

#[async_trait]
pub trait Connection {
    fn send(&self, msg: Message) -> Result<(), ConnError>;
    async fn recv(&self) -> Message;
}

/// Represents a connection error.
pub enum ConnError {
    Send(String),
    Recv(String),
}

impl From<futures::channel::mpsc::SendError> for ConnError {
    fn from(value: futures::channel::mpsc::SendError) -> Self {
        Self::Send(value.to_string())
    }
}
