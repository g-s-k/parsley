use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use super::{Cont, Env, Ns, Primitive, Proc, Result, SExp};

mod base;
mod core;
// mod math;
mod write;

/// Evaluation context for LISP expressions.
///
/// ## Note
/// `Context::default()` only provides *very* basic utilities. To obtain an
/// evaluation context with useful functions available, use
/// [`Context::base()`](#method.base).
///
/// ## Some implementation details
/// `Context` maintains separate environments for "core" (special forms, etc.),
/// "lang" (basic functions, vectors, and more), and "user" definitions. Most of
/// the provided methods operate on the "user" environment, as the intended use
/// case keeps the other environments immutable once they have been initialized.
pub struct Context {
    core: Ns,
    cont: Rc<RefCell<Cont>>,
    /// You can `insert` additional definitions here to make them available
    /// throughout the runtime. These definitions will not go out of scope
    /// automatically, but can be overridden (see [`get`](#method.get) for
    /// semantic details).
    pub lang: Ns,
    out: Option<String>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            core: Self::core(),
            cont: Cont::default().into_rc(),
            lang: Ns::new(),
            out: None,
        }
    }
}

impl Context {
    /// Add a new, nested scope.
    ///
    /// See [Context::pop](#method.pop) for a usage example.
    pub fn push(&mut self) {
        self.cont.borrow_mut().push();
    }

    /// Remove the most recently added scope.
    ///
    /// If the stack height is 1, all definitions will be cleared, and the
    /// global scope will be replaced with an empty one.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::default();
    ///
    /// assert_eq!(ctx.get("x"), None);
    /// ctx.push();
    /// ctx.define("x", SExp::Null);
    /// assert_eq!(ctx.get("x"), Some(SExp::Null));
    /// ctx.pop();
    /// assert_eq!(ctx.get("x"), None);
    /// ```
    pub fn pop(&mut self) {
        self.cont.borrow_mut().pop();
    }

    /// Create a new definition in the current scope.
    pub fn define(&mut self, key: &str, value: SExp) {
        self.cont.borrow().env().define(key, value)
    }

    /// Get the definition for a symbol in the execution environment.
    ///
    /// Returns `None` if no definition is found.
    ///
    /// # Override semantics
    /// This method searches for a definition in the following order:
    ///
    ///   1. The core language
    ///   2. User definitions, starting from the most recent scope and working
    ///      backward to the top-level
    ///   3. [Language-level definitions](#structfield.lang)
    ///
    /// What this means is that definitions populated in the `lang` field can be
    /// overridden inside the runtime (e.g. in a REPL), but special form keywords
    /// cannot. For example, we can `(define null "foo")`, but we cannot
    /// `(set! and or)`.
    ///
    /// # Examples
    /// ```
    /// let ctx = parsley::Context::default(); // only core definitions included
    /// assert!(ctx.get("potato").is_none());
    /// ```
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::default();
    ///
    /// ctx.define("x", SExp::from(3));
    /// assert_eq!(ctx.get("x"), Some(SExp::from(3)));
    /// ```
    pub fn get(&self, key: &str) -> Option<SExp> {
        // first check core (reserved keywords)
        if let Some(exp) = self.core.get(key) {
            return Some(exp.clone());
        }

        // then the environment stack
        if let Some(exp) = self.cont.borrow().env().get(key) {
            return Some(exp);
        }

        // then check the stdlib
        if let Some(exp) = self.lang.get(key) {
            return Some(exp.clone());
        }

        // otherwise fail
        None
    }

    /// Re-bind an existing definition to a new value.
    ///
    /// Returns `Ok` if an existing definition was found and updated. Returns
    /// `Err` if no definition exists.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::default();
    ///
    /// assert!(ctx.set("x", SExp::from(false)).is_err());    // Err, because x is not yet defined
    /// ctx.define("x", SExp::from(3));                       // define x
    /// assert_eq!(ctx.get("x"), Some(SExp::from(3)));        // check that its value is 3
    /// assert!(ctx.set("x", SExp::from("potato")).is_ok());  // Ok because x is now defined
    /// assert_eq!(ctx.get("x"), Some(SExp::from("potato"))); // check that its value is now "potato"
    /// ```
    pub fn set(&mut self, key: &str, value: SExp) -> Result {
        self.cont.borrow().env().set(key, value)
    }

    /// Push a new partial continuation with an existing environment.
    pub(super) fn use_env(&mut self, envt: Rc<Env>) {
        self.cont.borrow_mut().set_env(envt);
    }

    /// Push a new partial continuation onto the stack.
    pub(super) fn push_cont(&mut self) {
        self.cont = Cont::from(&self.cont).into_rc();
    }

    /// Pop the most recent partial continuation off of the stack.
    pub(super) fn pop_cont(&mut self) {
        let new = self.cont.borrow().parent().unwrap_or_default();
        self.cont = new;
    }

    fn eval_args(&mut self, args: SExp) -> Result {
        args.into_iter().map(|a| self.eval(a)).collect()
    }

    pub(super) fn eval_defer(&mut self, body: &SExp) -> Result {
        let mut result = Ok(SExp::Atom(Primitive::Undefined));

        let mut i = body.iter().peekable();

        while let Some(expr) = i.next() {
            if i.peek().is_some() {
                result = self.eval(expr.to_owned());
            } else {
                result = Ok(self.defer(expr.to_owned()))
            }

            if result.is_err() {
                break;
            }
        }
        result
    }

    /// Run a code snippet in an existing `Context`.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::base();
    ///
    /// assert!(ctx.run("x").is_err());
    /// assert!(ctx.run("(define x 6)").is_ok());
    /// assert_eq!(ctx.run("x").unwrap(), SExp::from(6));
    /// ```
    pub fn run(&mut self, expr: &str) -> Result {
        self.eval(expr.parse::<SExp>()?)
    }

    /// Evaluate an S-Expression in a context.
    ///
    /// The context will retain any definitions bound during evaluation
    /// (e.g. `define`, `set!`).
    ///
    /// # Examples
    /// ```
    /// use parsley::prelude::*;
    /// let result = Context::base().eval(
    ///     sexp![SExp::sym("eq?"), 0, 1]
    /// );
    /// assert_eq!(result.unwrap(), SExp::from(false));
    /// ```
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::base();
    ///
    /// let exp1 = sexp![SExp::sym("define"), SExp::sym("x"), 10];
    /// let exp2 = SExp::sym("x");
    ///
    /// ctx.eval(exp1);
    /// assert_eq!(ctx.eval(exp2).unwrap(), SExp::from(10));
    /// ```
    pub fn eval(&mut self, mut expr: SExp) -> Result {
        use super::Error::{NotAProcedure, NullList, UndefinedSymbol};
        use super::Func::Tail;
        use super::Primitive::{Procedure, Symbol, Undefined};
        use super::SExp::{Atom, Null, Pair};

        self.push_cont();

        let res = loop {
            expr = match expr {
                // cannot evaluate null
                Null => break Err(NullList),
                // check if symbol is defined
                Atom(Symbol(sym)) => match self.get(&sym) {
                    None | Some(Atom(Undefined)) => {
                        break Err(UndefinedSymbol { sym });
                    }
                    Some(exp) => exp,
                },
                // continue evaluation
                Atom(Procedure(Proc {
                    func: Tail { body, envt },
                    ..
                })) => {
                    self.cont.borrow_mut().set_env(envt);
                    expr = body.deref().to_owned();
                    continue;
                }
                // cannot reduce further
                Atom(_) => break Ok(expr),
                // it's an application
                Pair { head, tail } => {
                    // evaluate the first element
                    match self.eval(*head)? {
                        // if it is indeed a procedure
                        Atom(Procedure(p)) => {
                            let args = if p.defer_eval() {
                                *tail
                            } else {
                                self.eval_args(*tail)?
                            };
                            // then apply it
                            p.apply(args, self)?
                        }
                        // otherwise complain
                        proc => {
                            break Err(NotAProcedure {
                                exp: proc.to_string(),
                            });
                        }
                    }
                }
            };

            // see if we need to evaluate again
            match expr {
                Atom(Procedure(ref p)) if p.is_tail() => continue,
                _ => break Ok(expr),
            }
        };

        self.pop_cont();
        res
    }
}
