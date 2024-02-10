use super::{ConnError, Connection};
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures_util::{
    future::{select, Either},
    SinkExt, StreamExt,
};
use std::collections::HashMap;

pub struct Manager {
    recv_from_channels: UnboundedReceiver<Message>,
    send_from_channel: UnboundedSender<Message>,
    channels: HashMap<String, UnboundedSender<Message>>,
}

impl Manager {
    /// Create a new manager of a connection.
    pub fn new() -> Self {
        let (send_from_channel, recv_from_channels) = mpsc::unbounded();

        Self {
            recv_from_channels,
            send_from_channel,
            channels: HashMap::new(),
        }
    }

    /// Subscribe to a channel from the connection.
    pub fn channel<T: ToString>(&mut self, channel: T) -> ConnectionChannel {
        let channel = channel.to_string();
        let (sender, reciever) = mpsc::unbounded();

        let conn_channel =
            ConnectionChannel::new(channel.clone(), self.send_from_channel.clone(), reciever);

        self.channels.insert(channel, sender);

        conn_channel
    }

    /// Starts the manager with the specified connection.
    pub async fn listen(&mut self, conn: &mut impl Connection) {
        loop {
            let recv_channel = self.recv_from_channels.next();
            let recv_conn = conn.recv();

            match select(recv_channel, recv_conn).await {
                Either::Left((data, _)) => {
                    if let Some(data) = data {
                        // TODO: error handling
                        _ = conn.send(data);
                    }
                }
                Either::Right((data, _)) => {
                    // TODO: Handle none
                    let data = data.unwrap();
                    if let Some(sender) = self.channels.get_mut(&data.channel) {
                        // TODO: error handling
                        _ = sender.send(data).await;
                    }
                }
            }
        }
    }
}

pub struct ConnectionChannel {
    channel: String,
    sender: UnboundedSender<Message>,
    receiver: UnboundedReceiver<Message>,
}

impl ConnectionChannel {
    pub fn new(
        channel: String,
        sender: UnboundedSender<Message>,
        receiver: UnboundedReceiver<Message>,
    ) -> Self {
        Self {
            channel,
            sender,
            receiver,
        }
    }
    // Sends a message to the manager to then send to the connection.
    /// Send a message across the connection.
    pub async fn send<T: ToString>(&mut self, data: T) -> Result<(), ConnError> {
        let data = data.to_string();

        let final_message = Message {
            channel: self.channel.clone(),
            data,
        };

        Ok(self.sender.send(final_message).await?)
    }

    // Recieves messages from the manager.
    /// Recieve messages from the connection.
    pub async fn recv(&mut self) -> Option<Message> {
        self.receiver.next().await
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    channel: String,
    pub data: String,
}
