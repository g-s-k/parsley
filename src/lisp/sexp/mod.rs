use std::rc::Rc;

#[macro_use]
mod from;

mod display;
mod eval;
mod iter;
mod parse;

use super::{utils, Context, Error, Primitive, Result};

use self::SExp::{Atom, Null, Pair};

/// An S-Expression. Can be parsed from a string via `FromStr`, or constructed
/// programmatically.
///
/// # Examples
/// ```
/// use parsley::SExp;
/// let null = "()".parse::<SExp>().unwrap();
/// assert_eq!(null, SExp::Null);
/// ```
/// ```
/// use parsley::SExp;
/// let parsed = "\"abcdefg\"".parse::<SExp>().unwrap();
/// assert_eq!(parsed, SExp::from("abcdefg"));
/// ```
#[derive(PartialEq, Clone)]
pub enum SExp {
    Null,
    Atom(Primitive),
    Pair { head: Box<SExp>, tail: Box<SExp> },
}

impl SExp {
    fn apply(self, ctx: &mut Context) -> Result {
        match self {
            Null | Atom(_) => Ok(self),
            Pair { head, tail } => match *head {
                Atom(Primitive::Procedure(proc)) => {
                    trace!("Applying a procedure to the arguments {}", tail);
                    proc(*tail)?.eval(ctx)
                }
                Atom(Primitive::CtxProcedure(proc)) => proc(*tail, ctx),
                Atom(Primitive::Symbol(sym)) => Err(Error::NotAProcedure {
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

    pub(super) fn car(&self) -> Result {
        trace!("Getting the car of {}", self);
        match self {
            Null => Err(Error::NullList),
            Atom(_) => Err(Error::NotAList {
                atom: self.to_string(),
            }),
            Pair { head, .. } => Ok((**head).clone()),
        }
    }

    pub(super) fn cdr(&self) -> Result {
        trace!("Getting the cdr of {}", self);
        match self {
            Null => Err(Error::NullList),
            Atom(_) => Err(Error::NotAList {
                atom: self.to_string(),
            }),
            Pair { tail, .. } => Ok((**tail).to_owned()),
        }
    }

    /// The natural way to build up a list - from the end to the beginning.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// use parsley::SExp::Null;
    ///
    /// let macro_list = sexp![SExp::sym("quote"), ()];
    /// let cons_list = Null.cons(Null).cons(SExp::sym("quote"));
    ///
    /// assert_eq!(macro_list, cons_list);
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
    /// use parsley::prelude::*;
    /// let mut ctx = Context::base();
    ///
    /// // A null list is an empty application
    /// assert!(SExp::Null.eval(&mut ctx).is_err());
    ///
    /// // The symbol `null` (defined in `Context::base`) creates a quoted null list
    /// let result = SExp::sym("null").eval(&mut ctx).unwrap();
    /// assert_eq!(result, SExp::Null);
    /// ```
    pub fn sym(sym: &str) -> Self {
        Atom(Primitive::Symbol(sym.to_string()))
    }

    /// Printable type for an expression.
    ///
    /// # Example
    /// ```
    /// use parsley::SExp;
    ///
    /// assert_eq!(SExp::Null.type_of(), "null");
    /// assert_eq!(SExp::from(3).type_of(), "number");
    /// assert_eq!(SExp::from(true).type_of(), "bool");
    /// assert_eq!(SExp::from((5,)).type_of(), "list");
    /// ```
    pub fn type_of(&self) -> &str {
        match self {
            Null => "null",
            Atom(p) => p.type_of(),
            Pair { .. } => "list",
        }
    }

    pub fn ctx_proc<F>(f: F) -> Self
    where
        F: Fn(Self, &mut Context) -> Result + 'static,
    {
        Atom(Primitive::CtxProcedure(Rc::new(f)))
    }
}
