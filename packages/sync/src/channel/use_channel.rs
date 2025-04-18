use async_broadcast::{InactiveReceiver, Receiver, SendError, Sender, TrySendError, broadcast};
use dioxus::prelude::*;
use uuid::Uuid;

/// Send and listen for messages between multiple components.
#[derive(Clone, Copy)]
pub struct UseChannel<MessageType: Clone + 'static> {
    id: Uuid,
    sender: Signal<Sender<MessageType>>,
    inactive_receiver: Signal<InactiveReceiver<MessageType>>,
}

impl<T: Clone> PartialEq for UseChannel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<MessageType: Clone + 'static> UseChannel<MessageType> {
    /// Tries to send a message to all listeners of the channel.
    pub fn try_send(&self, msg: impl Into<MessageType>) -> Result<(), TrySendError<MessageType>> {
        self.sender.peek().try_broadcast(msg.into()).map(|_| ())
    }

    /// Sends a message to all listeners of the channel.
    pub async fn send(&self, msg: impl Into<MessageType>) -> Result<(), SendError<MessageType>> {
        self.sender.peek().broadcast(msg.into()).await.map(|_| ())
    }

    /// Create a receiver for the channel.
    /// You probably want to use [`super::use_listen_channel()`].
    pub fn receiver(&mut self) -> Receiver<MessageType> {
        self.inactive_receiver.peek().clone().activate()
    }
}

/// Send and listen for messages between multiple components.
pub fn use_channel<MessageType: Clone + 'static>(size: usize) -> UseChannel<MessageType> {
    use_hook(|| {
        let id = Uuid::new_v4();
        let (sender, receiver) = broadcast::<MessageType>(size);
        UseChannel {
            id,
            sender: Signal::new(sender),
            inactive_receiver: Signal::new(receiver.deactivate()),
        }
    })
}
