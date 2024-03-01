use futures::channel::mpsc::{self, UnboundedReceiver};

pub struct Server {
    reciever: UnboundedReceiver<Message>,
}

impl Connection for Server {}
