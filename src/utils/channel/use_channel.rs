use async_broadcast::{broadcast, InactiveReceiver, Receiver, SendError, Sender};
use dioxus::prelude::ScopeState;
use uuid::Uuid;

/// Send and listen for messages between multiple components.
#[derive(Debug, Clone)]
pub struct UseChannel<MessageType: Clone> {
    id: Uuid,
    sender: Sender<MessageType>,
    inactive_receiver: InactiveReceiver<MessageType>,
}

impl<T: Clone> PartialEq for UseChannel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<MessageType: Clone> UseChannel<MessageType> {
    /// Sends a message to all listeners of the channel.
    pub async fn send(&self, msg: impl Into<MessageType>) -> Result<(), SendError<MessageType>> {
        self.sender.broadcast(msg.into()).await.map(|_| ())
    }

    /// Create a receiver for the channel.
    /// You probably want to use [`use_listen_channel`].
    pub fn receiver(&mut self) -> Receiver<MessageType> {
        self.inactive_receiver.clone().activate()
    }
}

/// Send and listen for messages between multiple components.
pub fn use_channel<MessageType: Clone + 'static>(
    cx: &ScopeState,
    size: usize,
) -> UseChannel<MessageType> {
    let id = cx.use_hook(Uuid::new_v4);
    let (sender, inactive_receiver) = cx.use_hook(|| {
        let (sender, receiver) = broadcast::<MessageType>(size);

        (sender, receiver.deactivate())
    });

    UseChannel {
        id: *id,
        sender: sender.clone(),
        inactive_receiver: inactive_receiver.clone(),
    }
}
