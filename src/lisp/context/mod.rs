use std::collections::HashMap;

use super::Primitive::Undefined;
use super::SExp::{self, Atom};
use super::{LispError, LispResult};

mod base;

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
    /// use parsley::prelude::*;
    /// let mut ctx = Context::default();
    ///
    /// ctx.define("x", SExp::from(3));
    /// assert_eq!(ctx.get("x"), Some(SExp::from(3)));
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
    /// use parsley::prelude::*;
    /// let mut ctx = Context::default();
    ///
    /// assert!(ctx.set("x", SExp::from(false)).is_err());    // Err, because x is not yet defined
    /// ctx.define("x", SExp::from(3));                       // define x
    /// assert_eq!(ctx.get("x"), Some(SExp::from(3)));        // check that its value is 3
    /// assert!(ctx.set("x", SExp::from("potato")).is_ok());  // Ok because x is now defined
    /// assert_eq!(ctx.get("x"), Some(SExp::from("potato"))); // check that its value is now "potato"
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
}
