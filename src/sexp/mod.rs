#[macro_use]
mod from;

mod display;
mod eval;
mod iter;
mod parse;

use super::{utils, Error, Primitive, Result};

use self::SExp::{Atom, Null, Pair, Vector};

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
    Vector(Vec<SExp>),
}

impl SExp {
    pub(super) fn split_car(self) -> ::std::result::Result<(Self, Self), Error> {
        match self {
            Null => Err(Error::NullList),
            Atom(_) | Vector(_) => Err(Error::NotAList {
                atom: self.to_string(),
            }),
            Pair { head, tail } => Ok((*head, *tail)),
        }
    }

    pub(super) fn car(self) -> Result {
        Ok(self.split_car()?.0)
    }

    pub(super) fn cdr(self) -> Result {
        Ok(self.split_car()?.1)
    }

    pub(super) fn set_car(&mut self, new: Self) -> Result {
        match self {
            Null => Err(Error::NullList),
            Atom(_) | Vector(_) => Err(Error::NotAList {
                atom: self.to_string(),
            }),
            Pair { head, .. } => {
                *head = Box::new(new);
                Ok(Atom(Primitive::Undefined))
            }
        }
    }

    pub(super) fn set_cdr(&mut self, new: Self) -> Result {
        match self {
            Null => Err(Error::NullList),
            Atom(_) | Vector(_) => Err(Error::NotAList {
                atom: self.to_string(),
            }),
            Pair { tail, .. } => {
                *tail = Box::new(new);
                Ok(Atom(Primitive::Undefined))
            }
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
    /// assert!(ctx.eval(SExp::Null).is_err());
    ///
    /// // The symbol `null` (defined in `Context::base`) creates a null when evaluated
    /// let result = ctx.run("null").unwrap();
    /// assert_eq!(result, SExp::Null);
    /// ```
    pub fn sym(sym: &str) -> Self {
        Atom(Primitive::Symbol(sym.to_string()))
    }

    pub(super) fn sym_to_str(&self) -> Option<&str> {
        if let Atom(Primitive::Symbol(s)) = self {
            Some(s)
        } else {
            None
        }
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
            Vector(_) => "vector",
        }
    }
}
