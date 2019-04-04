mod visitor;
use super::Visitor;

pub fn new() -> Result<Box<Visitor>> {
    let visitor = visitor::TokioVisitor::new()?;
    Ok(Box::new(visitor))
}