//! Utilities for writing LISP procedures in Rust.
//!
//! Reduce code duplication for type/arity checking and value packaging.

use std::rc::Rc;

use super::super::{Error, Func, Proc};
use super::Primitive::{self, Number};
use super::SExp::{self, Atom};


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
        Func::Pure(Rc::new(move |e| {
            let n = e.car()?;

            if let SExp::Atom(Primitive::Number(n)) = n {
                Ok((f(n)).into())
            } else {
                Err(Error::Type {
                    expected: "number",
                    given: n.type_of().to_string(),
                })
            }
        })),
        1,
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
        Func::Pure(Rc::new(move |expr| {
            let (arg0, tail) = expr.split_car()?;
            let arg1 = tail.car()?;

            match (arg0, arg1) {
                (Atom(Number(n0)), Atom(Number(n1))) => Ok((f(n0, n1)).into()),
                (Atom(Number(_)), e) | (e, _) => Err(Error::Type {
                    expected: "number",
                    given: e.type_of().to_string(),
                }),
            }
        })),
        2,
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
        (0,),
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
        (1,),
        name,
    ))
}

pub fn make_unary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| f(exp.car()?))),
        1,
        name,
    ))
}

pub fn make_binary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp, SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| {
            let (arg0, tail) = exp.split_car()?;

            f(arg0, tail.car()?)
        })),
        2,
        name,
    ))
}

pub fn make_ternary_expr<F>(f: F, name: Option<&str>) -> SExp
where
    F: Fn(SExp, SExp, SExp) -> crate::Result + 'static,
{
    SExp::from(Proc::new(
        Func::Pure(Rc::new(move |exp| {
            let (arg0, tail) = exp.split_car()?;
            let (arg1, tail) = tail.split_car()?;

            f(arg0, arg1, tail.car()?)
        })),
        3,
        name,
    ))
}
