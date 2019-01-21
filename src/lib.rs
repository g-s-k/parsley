//! A Scheme parsing and evaluation framework.
//!
//! # Example
//! ```
//! use parsley::run;
//!
//! assert_eq!(
//!     run("(null? '())").unwrap(),
//!     run("#t").unwrap()
//! );
//!
//! assert_eq!(
//!     run("(* (+ 3 4 5) (- 5 2))").unwrap(),
//!     run("36").unwrap()
//! );
//!
//! let expr = r#"
//! (define (sqr x) (* x x))
//! (define (sum-of-squares x y) (+ (sqr x) (sqr y)))
//! (sum-of-squares 3 4)
//! "#;
//! assert_eq!(
//!     run(expr).unwrap(),
//!     run("25").unwrap()
//! );
//! ```

#![feature(box_patterns)]
#![deny(clippy::pedantic)]

use std::collections::HashMap;

#[macro_use]
mod sexp;

mod context;
mod errors;
mod primitives;
mod proc;
mod utils;

pub use self::context::{utils as proc_utils, Context};
pub use self::errors::Error;
use self::primitives::Primitive;
pub use proc::{Arity, Func, Proc};
pub use self::sexp::SExp;

/// A shorthand Result type.
pub type Result = ::std::result::Result<SExp, Error>;

/// A type to represent an execution environment.
type Env = HashMap<String, SExp>;

/// Run a code snippet in the [base context](./struct.Context.html#method.base).
///
/// # Example
/// ```
/// use parsley::prelude::*;
///
/// assert!(run("x").is_err());
/// assert!(run("null").is_ok());
/// assert_eq!(run("null").unwrap(), SExp::Null);
/// ```
pub fn run(code: &str) -> Result {
    Context::base().run(code)
}

/// Quick access to the important stuff.
pub mod prelude {
    pub use super::{eval, run, sexp, Context, SExp};
}
