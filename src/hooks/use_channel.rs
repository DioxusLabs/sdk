use std::future::Future;

use dioxus::prelude::{to_owned, use_effect, use_state, ScopeState, UseState};
use tokio::sync::broadcast::{
    self,
    error::{RecvError, SendError},
    Receiver, Sender,
};
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
            receiver: self.sender.subscribe(),
        }
    }
}

impl<MessageType: Clone> UseChannel<MessageType> {
    pub fn send(&self, msg: impl Into<MessageType>) -> Result<(), SendError<MessageType>> {
        self.sender.send(msg.into()).map(|_| ())
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
    let sender = cx.use_hook(|| broadcast::channel::<MessageType>(size).0);

    UseChannel {
        id: *id,
        sender: sender.clone(),
        receiver: sender.subscribe(),
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
                    Err(RecvError::Lagged(_)) => {}
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