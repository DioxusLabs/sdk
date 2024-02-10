use super::{ConnError, Connection, Message};
use async_trait::async_trait;
use dioxus::hooks::{UnboundedReceiver, UnboundedSender};
use futures::channel::mpsc;
use futures_util::{SinkExt, StreamExt};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::MessageEvent;

pub struct WebsocketClient {
    reciever: UnboundedReceiver,
}

impl WebsocketClient {
    pub fn connect(url: &str) -> Self {
        //TODO: Handle errors
        let ws = web_sys::Websocket::new(url).unwrap();

        let (tx, rx) = mpsc::unbounded();

        let cl = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let string: String = text.into();
                // TODO: Handle error
                let val = serde_json::from_str::<Message>(&string).unwrap();
                // TODO: Handle error
                tx.unbounded_send(val).unwrap();
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(cl.as_ref().unchecked_ref()));
        cl.forget();

        Self {
            reciever: rx,
        }
    }
}

#[async_trait]
impl Connection for WebsocketClient {
    fn send(&self, msg: Message) -> Result<(), ConnError> {
        todo!()
    }

    async fn recv(&self) -> Message {
        self.reciever.next().await
    }
}
