use super::super::{Visitor};
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::prelude::*;
use futures::future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::ClientBuilder;
use websocket::client::async::ClientNew;
pub use websocket::OwnedMessage;
use serde_json;
use std::sync::{Arc, Mutex};

#[derive(Serialize,Deserialize)]
#[serde(tag = "connection_status", content = "c")]
pub enum ConnectionStatus {
    Error(ConnectionError),
    Ok,
}
pub struct TokioVisitor {
    connections:Vec<String>,
    runtime: tokio::runtime::Builder,
    proxy: std::sync::mpsc::channel::<String>
    on_message: Box<Fn(String)>,
    events: Arc<Mutex<Vec<String>>>
}
impl TokioVisitor{
    pub unsafe fn new() -> Result<Self>{
        TokioVisitor{
            connections:Vec::new(),
            runtime: tokio::runtime::Builder::new().build().unwrap(),
            proxy: std::sync::mpsc::channel::<String>(),
            events:  Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl Visitor for TokioVisitor{
    #[inline]
    unsafe fn create_connection(&mut self,param:String)->Result<()>{
        if (!self.connections.contain(&param)){
            let p = param.clone();
            let runner = ClientBuilder::new(&p).unwrap()
            .async_connect_insecure()
            .join(future::ok::<std::sync::mpsc::channel::Sender,std::sync::mpsc::channel::Sender>(self.proxy.1.clone()))
            .and_then(|((duplex, _), gui_c)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let _content = match msg {
                        OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                        OwnedMessage::Ping(d) => Some(OwnedMessage::Ping(d)),
                        OwnedMessage::Text(f) => {
                            gui_c.send(f).unwrap();
                            None
                        }
                        _ => None,
                    };
                    // ... and send that string _to_ the GUI.

                    Ok(())
                });
            let writer = rx
            .map_err(|()| unreachable!("rx can't fail"))
            .fold(to_server, |to_server, msg| {
                let h= msg.clone();
                 to_server.send(h)
            })
            .map(|_| ());

                // Use select to allow either the reading or writing half dropping to drop the other
                // half. The `map` and `map_err` here effectively force this drop.
                reader.select(writer).map(|_| ()).map_err(|(err, _)| err)
            });
             match self.runtime.run(runner) { //block_on
                    Ok(_) => {
                        println!("connected");
                        let g = serde_json::to_string(&ConnectionStatus::Ok).unwrap();
                        gui_c.clone().send(g).unwrap();
                        Ok(())
                    }
                    Err(_er) => {
                        let g = serde_json::to_string(&ConnectionStatus::Error(ConnectionError::CannotFindServer)).unwrap();
                        gui_c.clone().send(g).unwrap();
                        Err(ConnectionError::CannotFindServer)
                    }
                }
            self.connections.push(param.clone());
        }
    }

    #[inline]
    unsafe fn poll_events(&mut self, v: &mut Vec<String>) {
        let mut events = self.events.lock().unwrap();
        self.proxy.2.iter().exend(events.drain(..));
    }
}