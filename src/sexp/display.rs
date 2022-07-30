use super::Primitive::Symbol;
use super::SExp::{self, Atom, Null, Pair};
use std::fmt;

impl fmt::Debug for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Null => write!(f, "()",),
            Atom(a) => write!(f, "{:?}", a),
            Pair { head, tail } => match &**head {
                Atom(Symbol(q)) if q == "quote" => match &**tail {
                    Pair { head: h2, tail: t2 } if **t2 == Null => write!(f, "'{}", h2),
                    _ => write!(f, "'{}", tail),
                },
                _ => {
                    write!(f, "({:?}", head)?;
                    match &**tail {
                        Atom(a) => write!(f, " . {:?}", a)?,
                        null_or_pair => null_or_pair
                            .iter()
                            .try_for_each(|item| write!(f, " {:?}", item))?,
                    }
                    write!(f, ")")
                }
            },
        }
    }
}

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Null => write!(f, "()",),
            Atom(a) => write!(f, "{}", a),
            Pair { head, tail } => match &**head {
                Atom(Symbol(q)) if q == "quote" => match &**tail {
                    Pair { head: h2, tail: t2 } if **t2 == Null => write!(f, "'{}", h2),
                    _ => write!(f, "'{}", tail),
                },
                _ => {
                    write!(f, "({}", head)?;
                    match &**tail {
                        Atom(a) => write!(f, " . {}", a)?,
                        null_or_pair => null_or_pair
                            .iter()
                            .try_for_each(|item| write!(f, " {}", item))?,
                    }
                    write!(f, ")")
                }
            },
        }
    }
}
