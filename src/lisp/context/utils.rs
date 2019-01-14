//! Utilities for writing LISP procedures in Rust.
//!
//! Reduce code duplication for type/arity checking and value packaging.

use super::super::{Error, Primitive, SExp};

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
///     SExp::from((make_unary_numeric(times_six, None), (7,)))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(42),
/// );
/// ```
pub fn make_unary_numeric<T>(f: impl Fn(f64) -> T + 'static, name: Option<&str>) -> SExp
where
    T: Into<SExp>,
{
    SExp::proc(
        move |e| match e {
            SExp::Pair {
                head: box SExp::Atom(Primitive::Number(n)),
                ..
            } => Ok((f(n)).into()),
            _ => Err(Error::Type),
        },
        name,
    )
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
///     SExp::from((make_binary_numeric(my_gte, None), (555, (444,))))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(true),
/// );
/// ```
pub fn make_binary_numeric<T>(f: impl Fn(f64, f64) -> T + 'static, name: Option<&str>) -> SExp
where
    T: Into<SExp>,
{
    SExp::proc(
        move |e| match e {
            SExp::Pair {
                head: box SExp::Atom(Primitive::Number(n1)),
                tail:
                    box SExp::Pair {
                        head: box SExp::Atom(Primitive::Number(n2)),
                        ..
                    },
            } => Ok((f(n1, n2)).into()),
            _ => Err(Error::Type),
        },
        name,
    )
}

/// Make a variadic procedure that takes a list of numeric arguments and folds
/// the whole list.
///
/// # Note
/// The underlying numeric type is f64.
///
/// # Example
/// ```
/// use parsley::prelude::*;
/// use parsley::proc_utils::*;
///
/// let my_adder = |accumulator, current| accumulator + current;
/// let my_add_proc = make_fold_numeric(0., my_adder, None);
///
/// assert_eq!(
///     SExp::from((my_add_proc, vec![1, 2, 3, 4]))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(10),
/// );
/// ```
pub fn make_fold_numeric<F, T>(init: T, f: F, name: Option<&str>) -> SExp
where
    F: Fn(T, f64) -> T + 'static,
    T: Into<SExp> + Clone + 'static,
{
    SExp::proc(
        move |exp: SExp| match exp.into_iter().fold(Ok(init.to_owned()), |a, e| {
            if let Ok(val) = a {
                if let SExp::Atom(Primitive::Number(n)) = e {
                    Ok(f(val, n))
                } else {
                    Err(Error::Type)
                }
            } else {
                a
            }
        }) {
            Ok(v) => Ok(v.into()),
            Err(err) => Err(err),
        },
        name,
    )
}

/// Make a variadic procedure that takes a list of numeric arguments, reserves
/// the value of the first element as the initial accumulator, then folds the
/// rest of the list into a number.
///
/// # Note
/// The underlying numeric type is f64.
///
/// # Example
/// ```
/// use parsley::prelude::*;
/// use parsley::proc_utils::*;
///
/// let my_subtract = |accumulator, current| accumulator - current;
/// let my_sub_proc = make_fold_from0_numeric(my_subtract, None);
///
/// assert_eq!(
///     SExp::from((my_sub_proc, vec![1, 2, -3, 4]))
///         .eval(&mut Context::default()).unwrap(),
///     SExp::from(-2),
/// );
/// ```
pub fn make_fold_from0_numeric<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(f64, f64) -> f64 + 'static,
{
    SExp::proc(
        move |exp: SExp| {
            let mut i = exp.into_iter();
            if let Some(SExp::Atom(Primitive::Number(first))) = i.next() {
                match i.fold(Ok(first), |a, e| {
                    if let Ok(val) = a {
                        if let SExp::Atom(Primitive::Number(n)) = e {
                            Ok(f(val, n))
                        } else {
                            Err(Error::Type)
                        }
                    } else {
                        a
                    }
                }) {
                    Ok(v) => Ok(v.into()),
                    Err(err) => Err(err),
                }
            } else {
                Err(Error::Type)
            }
        },
        name,
    )
}

pub fn make_unary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp) -> crate::Result + 'static,
{
    SExp::proc(
        move |exp| match exp {
            SExp::Pair {
                head: box arg,
                tail: box SExp::Null,
            }
            | arg @ SExp::Atom(_)
            | arg @ SExp::Vector(_) => f(arg),
            SExp::Pair { .. } => Err(Error::Arity {
                expected: 1,
                given: 2,
            }),
            SExp::Null => Err(Error::Arity {
                expected: 1,
                given: 0,
            }),
        },
        name,
    )
}
