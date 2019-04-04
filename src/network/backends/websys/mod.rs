mod visitor;
use super::Visitor;

pub fn new() -> Result<Box<Visitor>> {
    let visitor = visitor::WebVisitor::new()?;
    Ok(Box::new(visitor))
}