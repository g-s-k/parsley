use std::fmt;
use std::rc::Rc;

use super::{LispResult, SExp};

mod eq;
mod from;

#[derive(Clone)]
pub enum Primitive {
    Void,
    Undefined,
    Boolean(bool),
    Character(char),
    Number(f64),
    String(String),
    Symbol(String),
    Procedure(Rc<dyn Fn(SExp) -> LispResult>),
}


impl fmt::Debug for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Void => write!(f, "#<void>"),
            Primitive::Undefined => write!(f, "#<undefined>"),
            Primitive::Boolean(b) => write!(f, "<boolean {}>", b),
            Primitive::Character(c) => write!(f, "#\\{}", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "{}", s),
            Primitive::Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Void => write!(f, "#<void>"),
            Primitive::Undefined => write!(f, ""),
            Primitive::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Primitive::Character(c) => write!(f, "#\\{}", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "{}", s),
            Primitive::Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}
