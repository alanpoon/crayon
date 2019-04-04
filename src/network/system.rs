use std::sync::{Arc, Mutex, RwLock};

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::errors::*;
use crate::math::prelude::Vector2;
use crate::utils::object_pool::ObjectPool;

use super::backends::{self, Visitor};

impl_handle!(ConnectionHandle);

pub trait NetworkListener {
    fn on(&mut self, v: &String) -> Result<()>;
}

/// Represents an OpenGL context and the window or environment around it.
pub struct NetworkSystem {
    lis: LifecycleListenerHandle,
    state: Arc<NetworkState>,
}

struct NetworkState {
    visitor: RwLock<Box<dyn Visitor>>,
    events: Mutex<Vec<String>>,
    listeners: Mutex<ObjectPool<ConnectionHandle, Arc<Mutex<dyn NetworkListener>>>>,
}

impl LifecycleListener for Arc<NetworkState> {
    fn on_pre_update(&mut self) -> crate::errors::Result<()> {
        // Polls events from window, and returns the iterator over them.
        let mut events = self.events.lock().unwrap();
        events.clear();

        let mut visitor = self.visitor.write().unwrap();
        visitor.poll_events(&mut events);

        Ok(())
    }

    fn on_post_update(&mut self) -> crate::errors::Result<()> {
        // Swaps the buffers in case of double or triple buffering.
        //
        // **Warning**: if you enabled vsync, this function will block until the next time the screen
        // is refreshed. However drivers can choose to override your vsync settings, which means that
        // you can't know in advance whether swap_buffers will block or not.
        self.visitor.read().unwrap().swap_buffers()?;
        Ok(())
    }
}
impl EventListener for Arc<NetworkState> {
    fn on(&mut self, v: &String) -> Result<(), failure::Error> {
        self.

        Ok(())
    }
}
impl Drop for NetworkSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lis);
    }
}

impl NetworkSystem {
    pub fn new()-> Result<Self>{
        let state = Arc::new(NetworkState {
            listeners: Mutex::new(ObjectPool::new()),
            events: Mutex::new(Vec::new()),
            visitor: RwLock::new(backends::new(params)?),
        });

        let window = NetworkSystem {
            state: state.clone(),
            lis: crate::application::attach(state),
        };

        Ok(window)
    }
    /// Creates a new `NetworkSystem` and initalize OpenGL context.
    pub fn create_connection(self,String) -> Result<ConnectionHandle> {
        let handle = self.state.listeners.write().unwrap().create(params);
        Ok(handle)
    }
    /// Adds a event listener.
    pub fn add_event_listener<T: NetworkListener + 'static>(&self, lis: T) -> ConnectionHandle {
        let lis = Arc::new(Mutex::new(lis));
        self.state.listeners.lock().unwrap().create(lis)
    }

    /// Removes a event listener from window.
    pub fn remove_event_listener(&self, handle: ConnectionHandle) {
        self.state.listeners.lock().unwrap().free(handle);
    }

}
