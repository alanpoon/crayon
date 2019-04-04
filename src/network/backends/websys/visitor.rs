use super::super::{Visitor};
use web_sys::WebSocket;
use wasm_bindgen::prelude::*;

use std::sync::{Arc, Mutex};
#[allow(dead_code)]
pub struct WebVisitor {
    connections:Vec<String>,
    on_message: Closure<FnMut(String)>,
    events: Arc<Mutex<Vec<String>>>
}

impl WebVisitor{

}

impl Visitor for WebVisitor{
    #[inline]
    unsafe fn create_connection(&mut self,param:String)->Result<()>{
        if (!self.connections.contain(&param)){
        let events = Arc::new(Mutex::new(Vec::new()));

        let on_message = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |evt: String| {
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };
        WebSocket::new(param).set_onmessage(on_message.as_ref().unchecked_ref()).unwrap();
        self.events = events;
        self.connections.push(param);
        }
        Ok(())
    }
    #[inline]
    fn poll_events(&mut self, v: &mut Vec<String>) {
        let mut events = self.events.lock().unwrap();
        v.extend(events.drain(..));
    }
}