use std::fmt;
use std::rc::Rc;
use std::string::String as CoreString;

use super::{LispResult, SExp};

use self::Primitive::*;

mod eq;
mod from;

#[derive(Clone)]
pub enum Primitive {
    Void,
    Undefined,
    Boolean(bool),
    Character(char),
    Number(f64),
    String(CoreString),
    Symbol(CoreString),
    Procedure(Rc<dyn Fn(SExp) -> LispResult>),
}

impl fmt::Debug for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Void => write!(f, "#<void>"),
            Undefined => write!(f, "#<undefined>"),
            Boolean(b) => write!(f, "<boolean {}>", b),
            Character(c) => write!(f, "#\\{}", c),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "{}", s),
            Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Void => write!(f, "#<void>"),
            Undefined => write!(f, ""),
            Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Character(c) => write!(f, "#\\{}", c),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "{}", s),
            Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}
