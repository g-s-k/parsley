//! Utilities for writing LISP procedures in Rust.
//!
//! Reduce code duplication for type/arity checking and value packaging.

use std::rc::Rc;

use super::super::{Error, Func, Primitive, Proc, SExp};

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
///     Context::base().eval(
///         sexp![make_unary_numeric(times_six, None), 7]
///     ).unwrap(),
///     SExp::from(42),
/// );
/// ```
pub fn make_unary_numeric<T>(f: impl Fn(f64) -> T + 'static, name: Option<&str>) -> SExp
where
    T: Into<SExp>,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |e| match e {
            SExp::Pair {
                head: box SExp::Atom(Primitive::Number(n)),
                ..
            } => Ok((f(n)).into()),
            SExp::Pair {
                head: box expr,
                tail: box SExp::Null,
            }
            | expr => Err(Error::Type {
                expected: "number",
                given: expr.type_of().to_string(),
            }),
        })),
        1,
        None,
        name,
    ))
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
///     Context::base().eval(
///         sexp![make_binary_numeric(my_gte, None), 555, 444]
///     ).unwrap(),
///     SExp::from(true),
/// );
/// ```
pub fn make_binary_numeric<T>(f: impl Fn(f64, f64) -> T + 'static, name: Option<&str>) -> SExp
where
    T: Into<SExp>,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |e| match e {
            SExp::Pair {
                head: box SExp::Atom(Primitive::Number(n1)),
                tail:
                    box SExp::Pair {
                        head: box SExp::Atom(Primitive::Number(n2)),
                        ..
                    },
            } => Ok((f(n1, n2)).into()),
            _ => Err(Error::Type {
                expected: "list",
                given: e.type_of().to_string(),
            }),
        })),
        2,
        None,
        name,
    ))
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
///     Context::base().eval(
///         sexp![my_add_proc, 1, 2, 3, 4]
///     ).unwrap(),
///     SExp::from(10),
/// );
/// ```
pub fn make_fold_numeric<F, T>(init: T, f: F, name: Option<&str>) -> SExp
where
    F: Fn(T, f64) -> T + 'static,
    T: Into<SExp> + Clone + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp: SExp| {
            match exp.into_iter().fold(Ok(init.to_owned()), |a, e| {
                if let Ok(val) = a {
                    if let SExp::Atom(Primitive::Number(n)) = e {
                        Ok(f(val, n))
                    } else {
                        Err(Error::Type {
                            expected: "number",
                            given: e.type_of().to_string(),
                        })
                    }
                } else {
                    a
                }
            }) {
                Ok(v) => Ok(v.into()),
                Err(err) => Err(err),
            }
        })),
        0,
        None,
        name,
    ))
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
///     Context::base().eval(
///         sexp![my_sub_proc, 1, 2, -3, 4]
///     ).unwrap(),
///     SExp::from(-2),
/// );
/// ```
pub fn make_fold_from0_numeric<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(f64, f64) -> f64 + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp: SExp| {
            let mut i = exp.into_iter();
            match i.next() {
                Some(SExp::Atom(Primitive::Number(first))) => {
                    match i.fold(Ok(first), |a, e| {
                        if let Ok(val) = a {
                            if let SExp::Atom(Primitive::Number(n)) = e {
                                Ok(f(val, n))
                            } else {
                                Err(Error::Type {
                                    expected: "number",
                                    given: e.type_of().to_string(),
                                })
                            }
                        } else {
                            a
                        }
                    }) {
                        Ok(v) => Ok(v.into()),
                        Err(err) => Err(err),
                    }
                }
                Some(other) => Err(Error::Type {
                    expected: "number",
                    given: other.type_of().to_string(),
                }),
                None => Err(Error::ArityMin {
                    expected: 1,
                    given: 0,
                }),
            }
        })),
        1,
        None,
        name,
    ))
}

pub fn make_unary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| match exp {
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
        })),
        1,
        None,
        name,
    ))
}

pub fn make_binary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp, SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| match exp {
            SExp::Pair {
                head: box arg0,
                tail:
                    box SExp::Pair {
                        head: box arg1,
                        tail: box SExp::Null,
                    },
            } => f(arg0, arg1),
            SExp::Pair {
                tail: box SExp::Null,
                ..
            }
            | SExp::Atom(_)
            | SExp::Vector(_) => Err(Error::Arity {
                expected: 2,
                given: 1,
            }),
            p @ SExp::Pair { .. } => Err(Error::Arity {
                expected: 2,
                given: p.iter().count(),
            }),
            SExp::Null => Err(Error::Arity {
                expected: 2,
                given: 0,
            }),
        })),
        2,
        None,
        name,
    ))
}

pub fn make_ternary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp, SExp, SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| match exp {
            SExp::Pair {
                head: box arg0,
                tail:
                    box SExp::Pair {
                        head: box arg1,
                        tail:
                            box SExp::Pair {
                                head: box arg2,
                                tail: box SExp::Null,
                            },
                    },
            } => f(arg0, arg1, arg2),
            other_variant => Err(Error::Arity {
                expected: 3,
                given: other_variant.iter().count(),
            }),
        })),
        3,
        None,
        name,
    ))
}
