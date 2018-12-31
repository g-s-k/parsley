//! A Scheme parsing and evaluation framework.
//!
//! # Example
//! ```
//! use parsley::{Context, SExp};
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

#[macro_use]
extern crate failure_derive;

mod lisp;

pub use self::lisp::*;
