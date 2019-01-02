//! A Scheme parsing and evaluation framework.
//!
//! # Example
//! ```
//! use parsley::prelude::*;
//! let mut ctx = Context::base();
//!
//! let expr = "(null? '())";
//! let value = "#t";
//! assert_eq!(
//!     expr.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
//!     value.parse::<SExp>().unwrap().eval(&mut ctx).unwrap()
//! );
//!
//! let expr = "(* (+ 3 4 5) (- 5 2))";
//! let value = "36";
//! assert_eq!(
//!     expr.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
//!     value.parse::<SExp>().unwrap().eval(&mut ctx).unwrap()
//! );
//!
//! let expr = r#"
//! (define (sqr x) (* x x))
//! (define (sum-of-squares x y) (+ (sqr x) (sqr y)))
//! (sum-of-squares 3 4)
//! "#;
//! let value = "25";
//! assert_eq!(
//!     expr.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
//!     value.parse::<SExp>().unwrap().eval(&mut ctx).unwrap()
//! );
//! ```

#![feature(box_patterns, box_syntax)]

#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate log;

mod lisp;
pub use self::lisp::*;

/// Quick access to the important stuff.
pub mod prelude {
    pub use super::{Context, LispError, SExp};
}
