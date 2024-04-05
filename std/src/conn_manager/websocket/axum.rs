use crate::conn_manager::{Connection, Message};
use futures::channel::mpsc::{self, UnboundedReceiver};

pub struct Server {
    reciever: UnboundedReceiver<Message>,
}

impl Connection for Server {
    fn send(&self, msg: crate::conn_manager::Message) -> Result<(), crate::conn_manager::ConnError> {
        todo!()
    }

    fn recv(&mut self) -> futures::prelude::stream::Next<'_, UnboundedReceiver<crate::conn_manager::Message>> {
        todo!()
    }
}
