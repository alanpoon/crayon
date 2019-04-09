use super::super::{Visitor};
use web_sys::WebSocket;
use wasm_bindgen::prelude::*;
use crate::errors::Result;

use wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct WebVisitor {
    connections:Vec<String>,
    events: Arc<Mutex<Vec<String>>>,
    on_message: Closure<FnMut(String)>,
    ws: Option<WebSocket>
}

impl WebVisitor{
    pub fn new() -> Result<Self>{
        let events = Arc::new(Mutex::new(Vec::new()));
        let on_message = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |evt: String| {
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };
        Ok(WebVisitor{
            connections: Vec::new(),
            events: events,
            on_message:on_message,
            ws: None
        })
    }
}

impl Visitor for WebVisitor{
    #[inline]
    fn create_connection(&mut self,param:String)->Result<()>{
        if !self.connections.contains(&param){
        let ws = WebSocket::new_with_str_sequence(&param,&JsValue::from_str("rust-websocket")).unwrap();
        ws.set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
        self.ws = Some(ws);
        self.connections.push(param);
        }
        Ok(())
    }
    #[inline]
    fn poll_events(&mut self, v: &mut Vec<String>) {
        let mut events = self.events.lock().unwrap();
        v.extend(events.drain(..));
    }
    #[inline]
    fn send(&mut self,v:String){
        if let Some(ws) = &self.ws{
            ws.send_with_str(&v).unwrap();
        }
    }
}