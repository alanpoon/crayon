use super::super::{Visitor};
use web_sys::WebSocket;
use wasm_bindgen::prelude::*;
use crate::errors::Result;

use wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct WebVisitor {
    connections:Vec<String>,
    events: Arc<Mutex<Vec<String>>>
}

impl WebVisitor{
    pub fn new() -> Result<Self>{
        Ok(WebVisitor{
            connections: Vec::new(),
            events: Arc::new(Mutex::new(Vec::new()))
        })
    }
}

impl Visitor for WebVisitor{
    #[inline]
    fn create_connection(&mut self,param:String)->Result<()>{
        if !self.connections.contains(&param){
        let events = Arc::new(Mutex::new(Vec::new()));
        let on_message = {
            let clone = events.clone();
            Closure::wrap(Box::new(move |evt: String| {
                clone.lock().unwrap().push(evt);
            }) as Box<FnMut(_)>)
        };
        WebSocket::new(&param).unwrap().set_onmessage(Some(on_message.as_ref().unchecked_ref()));
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