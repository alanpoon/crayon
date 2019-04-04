use super::super::{Visitor};
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::ClientBuilder;
use websocket::client::async::ClientNew;
pub use websocket::OwnedMessage;
use serde_json;
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
}
impl TokioVisitor{
    pub unsafe fn new() -> Result<Self>{
        TokioVisitor{
            connections:Vec::new(),
            runtime: tokio::runtime::Builder::new().build().unwrap(),
            proxy: std::sync::mpsc::channel::<String>()
        }
    }
    pub poll(self) -> Vec<Connection>{
        let mut c = vec![];
        while let Ok(s) = self.proxy.2.try_recv() {
            c.push(s)
        }
        return c;
    } 
}

impl Visitor for TokioVisitor{
    #[inline]
    fn create_connection(param:String)->Result<()>{
        if (!self.connections.contain(&param)){
            let p = param.clone();
            let gui_c = proxy.1.clone();
            let runner = ClientBuilder::new(&p).unwrap()
            .async_connect_insecure()
            .and_then(move |(duplex, _)| {
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
            match self.runtime.run(runner) {
                    Ok(_) => {
                        println!("connected");
                        let g = serde_json::to_string(&ConnectionStatus::Ok).unwrap();
                        proxy.1.clone().send(OwnedMessage::Text(g)).unwrap();
                        Ok(())
                    }
                    Err(_er) => {
                        let g = serde_json::to_string(&ConnectionStatus::Error(ConnectionError::CannotFindServer)).unwrap();
                        proxy.1.clone().send(OwnedMessage::Text(g)).unwrap();
                        Err(ConnectionError::CannotFindServer)
                    }
                }
            self.connections.push(param.clone());
        }
    }
    fn connections()->Vec<Connection>{
        self.connections.
    }
}