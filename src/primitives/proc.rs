use std::rc::Rc;

use super::super::{Context, Result, SExp};

#[derive(Clone)]
pub enum Procedure {
    Basic(Rc<dyn Fn(SExp) -> Result>),
    Ctx(Rc<dyn Fn(&mut Context, SExp) -> Result>),
}
