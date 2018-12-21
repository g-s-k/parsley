use std::collections::HashMap;
use std::rc::Rc;

use super::as_atom::AsAtom;
use super::Primitive::{Number, Procedure};
use super::SExp::{self, Atom};
use super::{LispError, NULL};

/// Evaluation context for LISP expressions.
///
/// # Note
/// `Context::default()` does not provide any definitions. To obtain an
/// evaluation context with useful functions available, use
/// [`Context::base()`](#method.base).
#[derive(Debug, Clone)]
pub struct Context(Vec<HashMap<String, SExp>>);

impl Default for Context {
    fn default() -> Self {
        Context(vec![HashMap::new()])
    }
}

impl Context {
    /// Add a new, nested scope.
    pub fn push(&self) -> Self {
        let mut copy = self.clone();
        copy.0.push(HashMap::new());
        copy
    }

    /// Create a new definition in the current scope.
    pub fn define(&mut self, key: &str, value: SExp) {
        let num_frames = self.0.len();
        self.0[num_frames - 1].insert(key.to_string(), value);
    }

    /// Get the most recent definition for a symbol.
    ///
    /// This starts at the current scope and walks upward toward the global
    /// scope until it finds a match. If no match is found, it returns `None`.
    ///
    /// # Examples
    /// ```
    /// let ctx = parsley::Context::default(); // no definitions included
    /// assert!(ctx.get("potato").is_none());
    /// ```
    /// ```
    /// use parsley::{Context, AsAtom};
    /// let mut ctx = Context::default();
    /// ctx.define("x", 3_f64.as_atom());
    /// assert_eq!(ctx.get("x"), Some(3_f64.as_atom()));
    /// ```
    pub fn get(&self, key: &str) -> Option<SExp> {
        match self.0.iter().rev().find_map(|w| w.get(key)) {
            Some(exp) => Some(exp.clone()),
            _ => None,
        }
    }

    /// Set an existing definition to a new value.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, AsAtom};
    /// let mut ctx = Context::default();
    /// ctx.define("x", 3_f64.as_atom());
    /// assert_eq!(ctx.get("x"), Some(3_f64.as_atom()));
    /// ctx.set("x", "potato".as_atom());
    /// assert_eq!(ctx.get("x"), Some("potato".as_atom()));
    /// ```
    pub fn set(&mut self, key: &str, value: SExp) {
        for frame in self.0.iter_mut().rev() {
            if frame.contains_key(key) {
                frame.insert(key.to_string(), value);
                break;
            }
        }
    }

    /// Base context - defines a number of useful functions and constants for
    /// use in the runtime.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, NULL};
    /// let ctx = Context::base();
    /// assert_eq!(ctx.get("null").unwrap(), NULL);
    /// println!("{}", ctx.get("null?").unwrap()); // "#<procedure>"
    /// println!("{}", ctx.get("eq?").unwrap());   // "#<procedure>"
    /// println!("{}", ctx.get("+").unwrap());     // "#<procedure>"
    /// ```
    pub fn base() -> Self {
        let mut ret = Self::default();

        ret.define(
            "eq?",
            Atom(Procedure(Rc::new(|v| Ok((v[0] == v[1]).as_atom())))),
        );
        ret.define(
            "null?",
            Atom(Procedure(Rc::new(|v| Ok(v[0].is_null().as_atom())))),
        );
        ret.define("null", NULL);
        ret.define(
            "cons",
            Atom(Procedure(Rc::new(|v| {
                Ok(SExp::cons(v[0].to_owned(), v[1].to_owned()))
            }))),
        );
        ret.define("car", Atom(Procedure(Rc::new(|v| v[0].car()))));
        ret.define("cdr", Atom(Procedure(Rc::new(|v| v[0].cdr()))));
        ret.define(
            "+",
            Atom(Procedure(Rc::new(|v| {
                v.iter().fold(Ok(0_f64.as_atom()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n + na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }))),
        );
        ret.define(
            "-",
            Atom(Procedure(Rc::new(|v| {
                v.iter().skip(1).fold(Ok(v[0].clone()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(na - n)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }))),
        );
        ret.define(
            "*",
            Atom(Procedure(Rc::new(|v| {
                v.iter().fold(Ok(1_f64.as_atom()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n * na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }))),
        );
        ret.define(
            "/",
            Atom(Procedure(Rc::new(|v| {
                v.iter().skip(1).fold(Ok(v[0].clone()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(na / n)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }))),
        );

        ret
    }
}
