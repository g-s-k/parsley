//! Utilities for writing LISP procedures in Rust.
//!
//! Reduce code duplication for type/arity checking and value packaging.

use super::super::{LispError, Primitive, SExp};

/// Make a procedure that takes one numeric argument.
///
/// # Note
/// The underlying numeric type is f64.
///
/// # Example
/// ```
/// use parsley::prelude::*;
/// use parsley::proc_utils::*;
///
/// let times_six = |x| x * 6.;
///
/// assert_eq!(
///     SExp::from((make_unary_numeric(times_six), (7,)))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(42),
/// );
/// ```
pub fn make_unary_numeric<T>(f: impl Fn(f64) -> T + 'static) -> SExp
where
    T: Into<SExp>,
{
    (move |e| match e {
        SExp::Pair {
            head: box SExp::Atom(Primitive::Number(n)),
            ..
        } => Ok((f(n)).into()),
        _ => Err(LispError::TypeError),
    })
    .into()
}

/// Make a procedure that takes two numeric arguments.
///
/// # Note
/// The underlying numeric type is f64.
///
/// # Example
/// ```
/// use parsley::prelude::*;
/// use parsley::proc_utils::*;
///
/// let my_gte = |a, b| a >= b;
///
/// assert_eq!(
///     SExp::from((make_binary_numeric(my_gte), (555, (444,))))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(true),
/// );
/// ```
pub fn make_binary_numeric<T>(f: impl Fn(f64, f64) -> T + 'static) -> SExp
where
    T: Into<SExp>,
{
    (move |e| match e {
        SExp::Pair {
            head: box SExp::Atom(Primitive::Number(n1)),
            tail:
                box SExp::Pair {
                    head: box SExp::Atom(Primitive::Number(n2)),
                    ..
                },
        } => Ok((f(n1, n2)).into()),
        _ => Err(LispError::TypeError),
    })
    .into()
}
