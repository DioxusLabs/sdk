use std::future::Future;

use async_broadcast::{broadcast, Receiver, RecvError, SendError, Sender};
use dioxus::prelude::{to_owned, use_effect, use_state, ScopeState, UseState};
use uuid::Uuid;

#[derive(Debug)]
pub struct UseChannel<MessageType: Clone> {
    id: Uuid,
    sender: Sender<MessageType>,
    receiver: Receiver<MessageType>,
}

impl<T: Clone> PartialEq for UseChannel<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<MessageType: Clone> Clone for UseChannel<MessageType> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sender: self.sender.clone(),
            receiver: self.sender.new_receiver(),
        }
    }
}

impl<MessageType: Clone> UseChannel<MessageType> {
    pub async fn send(&self, msg: impl Into<MessageType>) -> Result<(), SendError<MessageType>> {
        self.sender.broadcast(msg.into()).await.map(|_| ())
    }

    pub async fn recv(&mut self) -> Result<MessageType, RecvError> {
        self.receiver.recv().await
    }
}

/// Create a bounded, multi-producer, multi-consumer channel where each sent value is broadcasted to all active receivers
pub fn use_channel<MessageType: Clone + 'static>(
    cx: &ScopeState,
    size: usize,
) -> UseChannel<MessageType> {
    let id = cx.use_hook(Uuid::new_v4);
    let (sender, receiver) = cx.use_hook(|| broadcast::<MessageType>(size));

    UseChannel {
        id: *id,
        sender: sender.clone(),
        receiver: receiver.clone(),
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum ChannelListenerState {
    Running,
    Stopped,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UseListenChannel {
    listener_state: UseState<ChannelListenerState>,
}

impl UseListenChannel {
    /// Stop the listener
    pub fn stop(&self) {
        if *self.listener_state.get() == ChannelListenerState::Running {
            self.listener_state.set(ChannelListenerState::Stopped);
        }
    }

    /// Resume the listener
    pub fn resume(&self) {
        if *self.listener_state.get() == ChannelListenerState::Stopped {
            self.listener_state.set(ChannelListenerState::Running);
        }
    }
}

/// Create a listener to a channel.
pub fn use_listen_channel<MessageType: Clone + 'static, Handler>(
    cx: &ScopeState,
    channel: &UseChannel<MessageType>,
    action: impl Fn(MessageType) -> Handler + 'static,
) -> UseListenChannel
where
    Handler: Future<Output = ()> + 'static,
{
    let listener_state = use_state(cx, || ChannelListenerState::Running);

    use_effect(cx, (listener_state,), move |(listener_state,)| {
        to_owned![listener_state, channel];
        async move {
            if *listener_state.current() == ChannelListenerState::Stopped {
                return;
            }
            let action = Box::new(action);
            loop {
                match channel.recv().await {
                    Ok(msg) => {
                        if *listener_state.current() == ChannelListenerState::Running {
                            action(msg).await;
                        }
                    }
                    Err(RecvError::Closed) => {
                        break;
                    }
                    Err(RecvError::Overflowed(_)) => {
                        log::info!("Channel overflowed.");
                    }
                }
                if *listener_state.current() == ChannelListenerState::Stopped {
                    break;
                }
            }
        }
    });

    UseListenChannel {
        listener_state: listener_state.clone(),
    }
}
