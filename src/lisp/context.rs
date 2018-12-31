use std::collections::HashMap;
use std::rc::Rc;

use quicli::prelude::*;

use super::as_atom::AsAtom;
use super::Primitive::{Character, Number, Procedure, String as LispString, Undefined};
use super::SExp::{self, Atom, Null, Pair};
use super::{LispError, LispResult};

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
        trace!("Creating a new scope.");
        self.0.push(HashMap::new());
    }

    /// Remove the most recently added scope.
    ///
    /// If the stack height is 1, all definitions will be cleared, and the
    /// global scope will be replaced with an empty one.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, SExp};
    /// let mut ctx = Context::default();
    /// assert_eq!(ctx.get("x"), None);
    /// ctx.push();
    /// ctx.define("x", SExp::Null);
    /// assert_eq!(ctx.get("x"), Some(SExp::Null));
    /// ctx.pop();
    /// assert_eq!(ctx.get("x"), None);
    /// ```
    pub fn pop(&mut self) {
        trace!("Leaving nested scope.");
        self.0.pop();

        if self.0.is_empty() {
            self.push();
        }
    }

    /// Create a new definition in the current scope.
    pub fn define(&mut self, key: &str, value: SExp) {
        trace!("Binding the symbol {} to the value {}.", key, value);
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
        trace!("Retrieving a definition for the key {}", key);
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
        trace!("Re-binding the symbol {} to the value {}", key, value);
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
    /// use parsley::{AsAtom, Context, SExp};
    /// let mut ctx = Context::base();
    ///
    /// let null_const = ctx.get("null").unwrap();
    /// let null_fn = ctx.get("null?").unwrap();
    /// assert_eq!(
    ///     SExp::Null.cons(null_const).cons(null_fn).eval(&mut ctx).unwrap(),
    ///     true.as_atom()
    /// );
    ///
    /// println!("{}", ctx.get("eq?").unwrap());   // "#<procedure>"
    /// println!("{}", ctx.get("+").unwrap());     // "#<procedure>"
    /// ```
    pub fn base() -> Self {
        let mut ret = Self::default();

        ret.define(
            "eq?",
            Atom(Procedure(Rc::new(|e| match e {
                Pair {
                    head: box elem1,
                    tail:
                        box Pair {
                            head: box elem2,
                            tail: box Null,
                        },
                } => Ok((elem1 == elem2).as_atom()),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }))),
        );
        ret.define(
            "null?",
            Atom(Procedure(Rc::new(|e| {
                trace!("{}", e);
                Ok((match e {
                    Pair {
                        head: box Null,
                        tail: box Null,
                    } => true,
                    _ => false,
                })
                .as_atom())
            }))),
        );
        ret.define("null", Null.cons(Null).cons(SExp::make_symbol("quote")));
        ret.define(
            "cons",
            Atom(Procedure(Rc::new(|e| match e {
                Pair {
                    head: box elem1,
                    tail:
                        box Pair {
                            head: box elem2,
                            tail: box Null,
                        },
                } => Ok(elem2.cons(elem1)),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }))),
        );
        ret.define("car", Atom(Procedure(Rc::new(|e| e.car()))));
        ret.define("cdr", Atom(Procedure(Rc::new(|e| e.cdr()))));
        ret.define(
            "+",
            Atom(Procedure(Rc::new(|v| {
                v.into_iter().fold(Ok(0_f64.as_atom()), |a, e| match e {
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
            Atom(Procedure(Rc::new(|e| match e {
                Null => Err(LispError::TypeError),
                a @ Atom(_) => Err(LispError::NotAList {
                    atom: a.to_string(),
                }),
                Pair {
                    head: box Atom(Number(n)),
                    tail,
                } => {
                    let mut state = n;

                    for exp in tail.into_iter() {
                        match exp {
                            Atom(Number(n2)) => state -= n2,
                            _ => return Err(LispError::TypeError),
                        }
                    }

                    Ok(Atom(Number(state)))
                }
                _ => Err(LispError::TypeError),
            }))),
        );
        ret.define(
            "*",
            Atom(Procedure(Rc::new(|v| {
                v.into_iter().fold(Ok(1_f64.as_atom()), |a, e| match e {
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
            Atom(Procedure(Rc::new(|e| match e {
                Null => Err(LispError::TypeError),
                a @ Atom(_) => Err(LispError::NotAList {
                    atom: a.to_string(),
                }),
                Pair {
                    head: box Atom(Number(n)),
                    tail,
                } => {
                    let mut state = n;

                    for exp in tail.into_iter() {
                        match exp {
                            Atom(Number(n2)) => state /= n2,
                            _ => return Err(LispError::TypeError),
                        }
                    }

                    Ok(Atom(Number(state)))
                }
                _ => Err(LispError::TypeError),
            }))),
        );
        ret.define(
            "string->list",
            Atom(Procedure(Rc::new(|e| match e {
                Pair {
                    head: box Atom(LispString(s)),
                    tail: box Null,
                } => Ok(s.chars().map(|c| Atom(Character(c))).rev().collect()),
                _ => Err(LispError::TypeError),
            }))),
        );
        ret.define(
            "list->string",
            Atom(Procedure(Rc::new(|e| match e {
                Pair { .. } => {
                    match e.into_iter().fold(Ok(String::new()), |s, e| match e {
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
            }))),
        );

        ret
    }
}
