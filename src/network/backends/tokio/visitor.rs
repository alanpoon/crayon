use super::super::{Visitor};
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::ClientBuilder;
use websocket::client::async::ClientNew;
pub use websocket::OwnedMessage;
pub use websocket::Message;
pub struct TokioVisitor {
    connections:Vec<ClientNew<TcpStream>>,
    core:
}
impl TokioVisitor{

}

impl Visitor for TokioVisitor{
    #[inline]
    fn create_connection(param:String)->Result<()>{
        let cb = ClientBuilder::new(&param).unwrap()
            .async_connect_insecure();
        self.connections.push(cb);
    }
    fn connections()->Vec<Connection>{
        self.connections.
    }
}