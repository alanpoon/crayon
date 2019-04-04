use super::Connection;

pub trait Visitor {
    unsafe fn create_connection(&mut self, params: String)
        -> Result<()>;
    unsafe fn poll_events(&mut self,v:&mut Vec<String>);
}

#[cfg(not(target_arch = "wasm32"))]
pub mod tokio;

#[cfg(not(target_arch = "wasm32"))]
pub fn new() -> Result<Box<Visitor>> {
    let visitor = unsafe { self::tokio::visitor::TokioVisitor::new()? };
    Ok(Box::new(visitor))
}

#[cfg(target_arch = "wasm32")]
pub mod websys;

#[cfg(target_arch = "wasm32")]
pub fn new() -> Result<Box<Visitor>> {
    let visitor = unsafe { websys::visitor::WebSysVisitor::new()? };
    Ok(Box::new(visitor))
}