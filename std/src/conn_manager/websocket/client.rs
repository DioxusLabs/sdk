use super::{ConnError, Connection, Message};
use futures::channel::mpsc::{self, UnboundedReceiver};
use futures_util::{stream::Next, StreamExt};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub struct Client {
    reciever: UnboundedReceiver<Message>,
    socket: WebSocket,
}

impl Client {
    pub fn connect(url: &str) -> Self {
        //TODO: Handle errors
        let ws = WebSocket::new(url).unwrap();
        let (tx, rx) = mpsc::unbounded();

        let cl = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let string: String = text.into();
                // TODO: Handle error
                let val = serde_json::from_str::<Message>(&string).unwrap();

                // Can't handle this error without panicking
                _ = tx.unbounded_send(val);
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(cl.as_ref().unchecked_ref()));
        cl.forget();

        Self {
            reciever: rx,
            socket: ws,
        }
    }
}

impl Connection for Client {
    fn send(&self, msg: Message) -> Result<(), ConnError> {
        // Serialize message
        let data = match serde_json::to_string(&msg) {
            Ok(d) => d,
            Err(e) => return Err(ConnError::Send(e.to_string())),
        };

        // Send it across the socket
        match self.socket.send_with_str(&data) {
            Ok(()) => Ok(()),
            Err(e) => Err(ConnError::Send(
                e.as_string()
                    .unwrap_or("failed to deserialize error".to_string()),
            )),
        }
    }

    fn recv(&mut self) -> Next<'_, UnboundedReceiver<Message>> {
        self.reciever.next()
    }
}
