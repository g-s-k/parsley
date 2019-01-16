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

mod context;
mod errors;
mod primitives;
mod sexp;
mod utils;

pub use self::context::{utils as proc_utils, Context};
pub use self::errors::Error;
use self::primitives::Primitive;
pub use self::sexp::SExp;

/// A shorthand Result type.
pub type Result = ::std::result::Result<SExp, Error>;

/// A type to represent an execution environment.
type Env = HashMap<String, SExp>;

/// Run a code snippet in an existing [Context](./struct.Context.html).
///
/// # Example
/// ```
/// use parsley::prelude::*;
/// let mut ctx = Context::base();
///
/// assert!(run_in("x", &mut ctx).is_err());
/// assert!(run_in("(define x 6)", &mut ctx).is_ok());
/// assert_eq!(run_in("x", &mut ctx).unwrap(), SExp::from(6));
/// ```
pub fn run_in(code: &str, ctx: &mut Context) -> Result {
    code.parse::<SExp>()?.eval(ctx)
}

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
    run_in(code, &mut Context::base())
}

/// Quick access to the important stuff.
pub mod prelude {
    pub use super::{eval, run, run_in, sexp, Context, SExp};
}
