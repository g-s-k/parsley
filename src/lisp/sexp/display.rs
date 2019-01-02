use super::SExp::{self, *};
use std::fmt;

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Null => write!(f, "'()",),
            Atom(a) => write!(f, "{}", a),
            Pair { head, tail } => {
                write!(f, "'({}", head)?;
                match &**tail {
                    Null => write!(f, ")"),
                    Atom(a) => write!(f, " . {})", a),
                    pair => {
                        let mut it = pair.to_owned().into_iter().peekable();
                        while let Some(element) = it.next() {
                            if it.peek().is_some() {
                                write!(f, " {}", element)?;
                            } else {
                                write!(f, " {})", element)?;
                            }
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}
