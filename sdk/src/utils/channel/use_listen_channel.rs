use std::{future::Future, rc::Rc};

use async_broadcast::RecvError;
use dioxus::prelude::*;

use super::UseChannel;

pub type UseListenChannelError = RecvError;

/// Create a messages listener for the given channel.
pub fn use_listen_channel<MessageType: Clone + 'static, Handler>(
    channel: &UseChannel<MessageType>,
    action: impl Fn(Result<MessageType, UseListenChannelError>) -> Handler + 'static,
) where
    Handler: Future<Output = ()> + 'static,
{
    let action = use_hook(|| Rc::new(action));
    use_memo(use_reactive(channel, move |mut channel| {
        to_owned![action];
        spawn(async move {
            let mut receiver = channel.receiver();
            loop {
                let message = receiver.recv().await;
                let message_err = message.clone().err();
                action(message).await;
                if message_err == Some(UseListenChannelError::Closed) {
                    break;
                }
            }
        })
    }));
}
