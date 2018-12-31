use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

use super::{utils, LispError, LispResult, SExp};

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

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Primitive::Void => {
                if let Primitive::Void = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Undefined => {
                if let Primitive::Undefined = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Boolean(b1) => {
                if let Primitive::Boolean(b2) = other {
                    b1 == b2
                } else {
                    false
                }
            }
            Primitive::Character(c1) => {
                if let Primitive::Character(c2) = other {
                    c1 == c2
                } else {
                    false
                }
            }
            Primitive::Number(n1) => {
                if let Primitive::Number(n2) = other {
                    n1 == n2
                } else {
                    false
                }
            }
            Primitive::String(s1) => {
                if let Primitive::String(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Symbol(s1) => {
                if let Primitive::Symbol(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Procedure(_) => false,
        }
    }
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
            Primitive::Symbol(s) => write!(f, "'{}", s),
            Primitive::Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Void => write!(f, "#<void>"),
            Primitive::Undefined => write!(f, "#<undefined>"),
            Primitive::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Primitive::Character(c) => write!(f, "#\\{}", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "'{}", s),
            Primitive::Procedure(_) => write!(f, "#<native code>"),
        }
    }
}

impl FromStr for Primitive {
    type Err = LispError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#t" => return Ok(Primitive::Boolean(true)),
            "#f" => return Ok(Primitive::Boolean(false)),
            _ => (),
        }

        if let Ok(num) = s.parse::<f64>() {
            return Ok(Primitive::Number(num));
        }

        if s.len() == 3 && s.starts_with("#\\") {
            return Ok(Primitive::Character(s.chars().nth(2).unwrap()));
        }

        if s.starts_with('"') && s.ends_with('"') {
            match utils::find_closing_delim(s.chars(), '"', '"') {
                Some(idx) if idx + 1 == s.len() => {
                    return Ok(Primitive::String(s.get(1..idx).unwrap().to_string()));
                }
                _ => (),
            }
        }

        if s.chars().all(utils::is_symbol_char) {
            return Ok(Primitive::Symbol(s.to_string()));
        }

        Err(LispError::SyntaxError { exp: s.to_string() })
    }
}
