mod system;

mod backends;
use self::system::NetworkSystem;
/// Setup the resource system.
pub(crate) unsafe fn setup() {
    debug_assert!(CTX.is_null(), "duplicated setup of resource system.");
    let ctx = NetworkSystem::new();
    CTX = Box::into_raw(Box::new(ctx));
}

/// Discard the resource system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut NetworkSystem));
    CTX = std::ptr::null();
}

pub struct Connection{
    message: String
}
/// Creates an connection
#[inline]
pub fn create_connection(params: String) -> Result<ConnectionHandle> {
    ctx().create_connection(params)
}
/// list all connections
#[inline]
pub fn connections()-> Vec<Connection>{
    ctx().connections()
}
/// Adds a event listener.
pub fn attach<T: EventListener + 'static>(lis: T) -> EventListenerHandle {
    ctx().add_event_listener(lis)
}

/// Removes a event listener from window.
pub fn detach(handle: EventListenerHandle) {
    ctx().remove_event_listener(handle)
}

mod ins {
    use super::system::NetworkSystem;

    pub static mut CTX: *const NetworkSystem = std::ptr::null();

    #[inline]
    pub fn ctx() -> &'static NetworkSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "Network system has not been initialized properly."
            );

            &*CTX
        }
    }
}
