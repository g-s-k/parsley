use super::Primitive::Symbol;
use super::SExp::{self, Atom, Null, Pair, Vector};
use std::fmt;

impl fmt::Debug for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Null => write!(f, "()",),
            Atom(a) => write!(f, "{:?}", a),
            Vector(v) => write!(
                f,
                "#({})",
                v.iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
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
                            .map(|item| write!(f, " {:?}", item))
                            .collect::<fmt::Result>()?,
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
            Vector(v) => write!(
                f,
                "#({})",
                v.iter()
                    .map(SExp::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
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
                            .map(|item| write!(f, " {}", item))
                            .collect::<fmt::Result>()?,
                    }
                    write!(f, ")")
                }
            },
        }
    }
}
