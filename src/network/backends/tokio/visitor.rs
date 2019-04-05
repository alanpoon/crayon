use super::super::{Visitor};
use crate::errors::Result;
use tokio;
use futures::future::Future;
use futures::future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::ClientBuilder;
pub use websocket::OwnedMessage;
use serde_json;
use std::sync::{Arc, Mutex};
#[derive(Serialize,Deserialize)]
#[serde(tag = "connection_status", content = "c")]
pub enum ConnectionStatus {
    Error(ConnectionError),
    Ok,
}
#[derive(Serialize,Deserialize)]
pub enum ConnectionError {
    NotConnectedToInternet,
    CannotFindServer,
    InvalidDestination,
}
pub struct TokioVisitor {
    connections:Vec<String>,
    proxy: (std::sync::mpsc::Sender<String>,std::sync::mpsc::Receiver<String>),
    server_proxy: (mpsc::Sender<String>,mpsc::Receiver<String>),
    events: Arc<Mutex<Vec<String>>>
}
impl TokioVisitor{
    pub fn new() -> Result<Self>{
        Ok(TokioVisitor{
            connections:Vec::new(),
            proxy: std::sync::mpsc::channel::<String>(),
            server_proxy: mpsc::channel::<String>(3),
            events:  Arc::new(Mutex::new(Vec::new()))
        })
    }
}

impl Visitor for TokioVisitor{
    #[inline]
    fn create_connection(&mut self,param:String)->Result<()>{
        if !self.connections.contains(&param){
            let runtime = tokio::runtime::Builder::new().build().unwrap();
            let p = param.clone();
            let f = future::join_all(vec![
                future::ok::<std::sync::mpsc::Sender<String>,websocket::result::WebSocketError>(self.proxy.0.clone()),
                future::ok::<std::sync::mpsc::Sender<String>,websocket::result::WebSocketError>(self.server_proxy.1),
                ClientBuilder::new(&p).unwrap().async_connect_insecure()
            ]);
            let f = f.and_then(|(((duplex, _), gui_c),rx)| {
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
            self.connections.push(param.clone());
            match runtime.block_on(runner) { //block_on
                Ok(_) => {
                    println!("connected");
                    let g = serde_json::to_string(&ConnectionStatus::Ok).unwrap();
                    self.proxy.0.clone().send(g).unwrap();
                    Ok(())
                }
                Err(_er) => {
                    let g = serde_json::to_string(&ConnectionStatus::Error(ConnectionError::CannotFindServer)).unwrap();
                    self.proxy.0.clone().send(g).unwrap();
                    Err(ConnectionError::CannotFindServer)
                }
            }
        }
    }

    #[inline]
    fn poll_events(&mut self, v: &mut Vec<String>) {
        let mut events = self.events.lock().unwrap();
        self.proxy.1.iter().exends(events.drain(..));
    }
}