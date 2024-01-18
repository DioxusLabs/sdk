use std::future::Future;

use async_broadcast::RecvError;
use dioxus::prelude::{use_effect, ScopeState};

use super::UseChannel;

pub type UseListenChannelError = RecvError;

/// Create a messages listener for the given channel.
pub fn use_listen_channel<MessageType: Clone + 'static, Handler>(
    cx: &ScopeState,
    channel: &UseChannel<MessageType>,
    action: impl Fn(Result<MessageType, UseListenChannelError>) -> Handler + 'static,
) where
    Handler: Future<Output = ()> + 'static,
{
    use_effect(cx, (channel,), move |(mut channel,)| {
        async move {
            let action = Box::new(action);
            let mut receiver = channel.receiver();

            loop {
                let message = receiver.recv().await;
                let message_err = message.clone().err();
                action(message).await;
                if message_err == Some(UseListenChannelError::Closed) {
                    break;
                }
            }
        }
    });
}
