use std::fmt;
use std::rc::Rc;
use std::string::String as CoreString;

use super::{Result, SExp};

use self::Primitive::{Boolean, Character, Number, Procedure, String, Symbol, Undefined, Void};

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
    Procedure(Rc<dyn Fn(SExp) -> Result>),
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

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Void, Void) | (Undefined, Undefined) => true,
            (Boolean(b1), Boolean(b2)) => b1 == b2,
            (Character(c1), Character(c2)) => c1 == c2,
            (Number(n1), Number(n2)) => n1 == n2,
            (String(s1), String(s2)) | (Symbol(s1), Symbol(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl Primitive {
    pub fn type_of(&self) -> &str {
        match self {
            Void => "void",
            Undefined => "undefined",
            Boolean(_) => "bool",
            Character(_) => "char",
            Number(_) => "number",
            String(_) => "string",
            Symbol(_) => "symbol",
            Procedure(_) => "procedure",
        }
    }
}
