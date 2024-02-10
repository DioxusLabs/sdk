pub mod manager;
use futures::channel::mpsc::UnboundedReceiver;
use futures_util::stream::Next;
pub use manager::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "conn_mgr_ws")] {
        pub mod websocket;
    }
}

pub trait Connection {
    fn send(&self, msg: Message) -> Result<(), ConnError>;
    fn recv(&mut self) -> Next<'_, UnboundedReceiver<Message>>;
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
