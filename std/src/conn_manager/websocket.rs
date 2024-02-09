use super::{ConnError, Connection, Message};
use async_trait::async_trait;

pub struct Websocket {}

impl Websocket {
    pub fn connect(url: &str) -> Self {
        todo!()
    }
}

#[async_trait]
impl Connection for Websocket {
    fn send(&self, msg: Message) -> Result<(), ConnError> {
        todo!()
    }

    async fn recv(&self) -> Message {
        todo!()
    }
}
