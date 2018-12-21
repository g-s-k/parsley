use std::collections::HashMap;
use std::rc::Rc;

use quicli::prelude::*;

use super::as_atom::AsAtom;
use super::Primitive::{Character, Number, Procedure, String as LispString, Undefined};
use super::SExp::{self, Atom, List};
use super::{LispError, LispResult, NULL};

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
    ///
    /// See [Context::pop](#method.pop) for a usage example.
    pub fn push(&mut self) {
        debug!("Creating a new scope.");
        self.0.push(HashMap::new());
    }

    /// Remove the most recently added scope.
    ///
    /// If the stack height is 1, all definitions will be cleared, and the
    /// global scope will be replaced with an empty one.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, NULL};
    /// let mut ctx = Context::default();
    /// assert_eq!(ctx.get("x"), None);
    /// ctx.push();
    /// ctx.define("x", NULL);
    /// assert_eq!(ctx.get("x"), Some(NULL));
    /// ctx.pop();
    /// assert_eq!(ctx.get("x"), None);
    /// ```
    pub fn pop(&mut self) {
        debug!("Leaving nested scope.");
        self.0.pop();

        if self.0.is_empty() {
            self.push();
        }
    }

    /// Create a new definition in the current scope.
    pub fn define(&mut self, key: &str, value: SExp) {
        debug!("Binding the symbol {} to the value {}.", key, value);
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
        debug!("Retrieving a definition.");
        match self.0.iter().rev().find_map(|w| w.get(key)) {
            Some(exp) => Some(exp.clone()),
            _ => None,
        }
    }

    /// Re-bind an existing definition to a new value.
    ///
    /// Returns `Ok` if an existing definition was found and updated. Returns
    /// `Err` if no definition exists.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, AsAtom};
    /// let mut ctx = Context::default();
    /// assert!(ctx.set("x", false.as_atom()).is_err());    // Err, because x is not yet defined
    /// ctx.define("x", 3_f64.as_atom());                   // define x
    /// assert_eq!(ctx.get("x"), Some(3_f64.as_atom()));    // check that its value is 3
    /// assert!(ctx.set("x", "potato".as_atom()).is_ok());  // Ok because x is now defined
    /// assert_eq!(ctx.get("x"), Some("potato".as_atom())); // check that its value is now "potato"
    /// ```
    pub fn set(&mut self, key: &str, value: SExp) -> LispResult {
        debug!("Re-binding a symbol.");
        for frame in self.0.iter_mut().rev() {
            if frame.contains_key(key) {
                frame.insert(key.to_string(), value);
                return Ok(Atom(Undefined));
            }
        }
        Err(LispError::UndefinedSymbol {
            sym: key.to_string(),
        })
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
        ret.define(
            "string->list",
            Atom(Procedure(Rc::new(|v| match v.len() {
                1 => match v[0] {
                    Atom(LispString(ref s)) => {
                        let mut elements = vec![SExp::make_symbol("quote")];
                        elements.push(List(s.chars().map(|c| Atom(Character(c))).collect()));
                        Ok(List(elements))
                    }
                    _ => Err(LispError::TypeError),
                },
                n_args => Err(LispError::TooManyArguments {
                    n_args,
                    right_num: 1,
                }),
            }))),
        );
        ret.define(
            "list->string",
            Atom(Procedure(Rc::new(|v| match v.len() {
                1 => match v[0] {
                    List(ref elems) => {
                        match elems.iter().fold(Ok(String::new()), |s, e| match e {
                            Atom(Character(ref c)) => {
                                if let Ok(st) = s {
                                    let mut stri = st;
                                    stri.push(*c);
                                    Ok(stri)
                                } else {
                                    s
                                }
                            }
                            _ => Err(LispError::TypeError),
                        }) {
                            Ok(s) => Ok(Atom(LispString(s))),
                            Err(err) => Err(err),
                        }
                    }
                    _ => Err(LispError::TypeError),
                },
                n_args => Err(LispError::TooManyArguments {
                    n_args,
                    right_num: 1,
                }),
            }))),
        );

        ret
    }
}
