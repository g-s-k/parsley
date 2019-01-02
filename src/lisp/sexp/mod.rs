mod display;
mod eval;
mod iter;
mod parse;

use super::{utils, Context, LispError, LispResult, Primitive};

use self::SExp::*;

/// An S-Expression. Can be parsed from a string via FromStr, or constructed
/// programmatically.
///
/// # Examples
/// ```
/// use parsley::SExp;
/// let null = "()".parse::<SExp>().unwrap();
/// assert_eq!(null, SExp::Null);
/// ```
/// ```
/// use parsley::{AsAtom, SExp};
/// let parsed = "\"abcdefg\"".parse::<SExp>().unwrap();
/// assert_eq!(parsed, "abcdefg".as_atom());
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum SExp {
    Null,
    Atom(Primitive),
    Pair { head: Box<SExp>, tail: Box<SExp> },
}

impl SExp {
    fn apply(self, ctx: &mut Context) -> LispResult {
        match self {
            Null | Atom(_) => Ok(self),
            Pair { head, tail } => match *head {
                Atom(Primitive::Procedure(proc)) => {
                    trace!("Applying a procedure to the arguments {}", tail);
                    proc(*tail)?.eval(ctx)
                }
                Atom(Primitive::Symbol(sym)) => Err(LispError::NotAProcedure {
                    exp: sym.to_string(),
                }),
                Pair {
                    head: proc,
                    tail: tail2,
                } => tail2.cons(proc.eval(ctx)?).eval(ctx),
                _ => Ok(tail.cons(*head)),
            },
        }
    }

    pub(super) fn car(&self) -> LispResult {
        trace!("Getting the car of {}", self);
        match self {
            Null => Err(LispError::NullList),
            Atom(_) => Err(LispError::NotAList {
                atom: self.to_string(),
            }),
            Pair { head, .. } => Ok((**head).clone()),
        }
    }

    pub(super) fn cdr(&self) -> LispResult {
        trace!("Getting the cdr of {}", self);
        match self {
            Null => Err(LispError::NullList),
            Atom(_) => Err(LispError::NotAList {
                atom: self.to_string(),
            }),
            Pair { tail, .. } => Ok((**tail).to_owned()),
        }
    }

    /// The natural way to build up a list - from the end to the beginning.
    ///
    /// # Example
    /// ```
    /// use parsley::SExp::{self, Null};
    ///
    /// let code = "(quote ())";
    /// let list = Null.cons(Null).cons(SExp::make_symbol("quote"));
    ///
    /// let parsed_code = code.parse::<SExp>().unwrap();
    /// assert_eq!(parsed_code, list);
    /// ```
    pub fn cons(self, exp: Self) -> Self {
        Pair {
            head: Box::new(exp),
            tail: Box::new(self),
        }
    }

    /// Convenience method to build a symbolic atom.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, SExp};
    /// let mut ctx = Context::base();
    ///
    /// // A null list is an empty application
    /// assert!(SExp::Null.eval(&mut ctx).is_err());
    ///
    /// // The symbol `null` (defined in `Context::base`) creates a null list
    /// let result = SExp::make_symbol("null").eval(&mut ctx);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), SExp::Null);
    /// ```
    pub fn make_symbol(sym: &str) -> Self {
        Atom(Primitive::Symbol(sym.to_string()))
    }
}
