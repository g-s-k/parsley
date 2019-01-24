use std::fmt;
use std::string::String as CoreString;

use super::proc::Proc;
use super::Ns;
use super::SExp;

use self::Primitive::{
    Boolean, Character, Env, Number, Procedure, String, Symbol, Undefined, Vector, Void,
};

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
    Env(Ns),
    Procedure(Proc),
    Vector(Vec<SExp>),
}

impl fmt::Debug for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Void => f.write_str("#<void>"),
            Undefined => f.write_str("#<undefined>"),
            Boolean(b) => f.write_str(if *b { "#t" } else { "#f" }),
            Character(c) => write!(f, "#\\{}", c),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "\"{}\"", s),
            Symbol(s) => write!(f, "{}", s),
            Env(_) => write!(f, "#<environment>"),
            Procedure(p) => write!(f, "{}", p),
            Vector(v) => write!(
                f,
                "#({})",
                v.iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Undefined | Void => Ok(()),
            Boolean(b) => f.write_str(if *b { "#t" } else { "#f" }),
            Character(c) => write!(f, "{}", c),
            Number(n) => write!(f, "{}", n),
            String(s) | Symbol(s) => f.write_str(s),
            Env(_) => write!(f, "#<environment>"),
            Procedure(p) => write!(f, "{}", p),
            Vector(v) => write!(
                f,
                "#({})",
                v.iter().map(SExp::to_string).collect::<Vec<_>>().join(" ")
            ),
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
            (Env(e1), Env(e2)) => e1 == e2,
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
            Env(_) => "environment",
            Procedure { .. } => "procedure",
            Vector(_) => "vector",
        }
    }
}
