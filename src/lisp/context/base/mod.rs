use super::super::Error;
use super::super::Primitive::{
    Character, CtxProcedure, Number, Procedure, String as LispString, Void,
};
use super::super::SExp::{self, Atom, Null, Pair, Vector};

use super::utils::*;
use super::Context;

mod tests;

macro_rules! define_with {
    ( $ctx:ident, $name:expr, $proc:expr, $tform:expr ) => {
        $ctx.define($name, $tform($proc, Some($name)))
    };
}

macro_rules! define_ctx {
    ( $ctx:ident, $name:expr, $proc:expr ) => {
        define_with!($ctx, $name, $proc, $crate::SExp::ctx_proc)
    };
}

macro_rules! define {
    ( $ctx:ident, $name:expr, $proc:expr ) => {
        $ctx.define($name, $crate::SExp::proc($proc, Some($name)))
    };
}

impl Context {
    /// Base context - defines a number of useful functions and constants for
    /// use in the runtime.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::base();
    ///
    /// let null_const = ctx.get("null").unwrap();
    /// let null_fn = ctx.get("null?").unwrap();
    /// assert_eq!(
    ///     sexp![null_fn, null_const].eval(&mut ctx).unwrap(),
    ///     SExp::from(true),
    /// );
    ///
    /// println!("{}", ctx.get("eq?").unwrap());   // "#<procedure>"
    /// println!("{}", ctx.get("+").unwrap());     // "#<procedure>"
    /// ```
    pub fn base() -> Self {
        let mut ret = Self::default();

        // The basics
        define_ctx!(ret, "eval", |e, c| e.car()?.eval(c)?.eval(c));
        define_ctx!(ret, "apply", SExp::do_apply);
        define_ctx!(ret, "and", SExp::eval_and);
        define_ctx!(ret, "begin", SExp::eval_begin);
        define_ctx!(ret, "case", SExp::eval_case);
        define_ctx!(ret, "cond", SExp::eval_cond);
        define_ctx!(ret, "define", SExp::eval_define);
        define_ctx!(ret, "if", SExp::eval_if);
        define_ctx!(ret, "lambda", |e, c| SExp::eval_lambda(e, c, false));
        define_ctx!(ret, "let", SExp::eval_let);
        define_ctx!(ret, "named-lambda", |e, c| SExp::eval_lambda(e, c, true));
        define_ctx!(ret, "or", SExp::eval_or);
        define_ctx!(ret, "quote", SExp::eval_quote);
        define_ctx!(ret, "set!", SExp::eval_set);
        define!(ret, "eq?", |e| match e {
            Pair {
                head: elem1,
                tail:
                    box Pair {
                        head: elem2,
                        tail: box Null,
                    },
            } => Ok((elem1 == elem2).into()),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        });
        define!(ret, "null?", |e| Ok((e == ((),).into()).into()));
        ret.define("null", (SExp::sym("quote"), ((),)).into());
        define!(ret, "void", |_| Ok(Atom(Void)));
        define!(ret, "list", Ok);
        define!(ret, "not", |e| Ok((e == (false,).into()).into()));
        define!(ret, "cons", |e| match e {
            Pair {
                head: elem1,
                tail:
                    box Pair {
                        head: elem2,
                        tail: box Null,
                    },
            } => Ok(Null.cons(elem2.cons(*elem1)).cons(SExp::sym("quote"))),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        });
        define_with!(ret, "car", SExp::car, make_unary_expr);
        define_with!(ret, "cdr", SExp::cdr, make_unary_expr);
        define_with!(
            ret,
            "type-of",
            |e| Ok(SExp::from(e.type_of())),
            make_unary_expr
        );

        // i/o
        define_ctx!(ret, "display", |e, c| SExp::do_print(e, c, false, false));
        define_ctx!(ret, "displayln", |e, c| SExp::do_print(e, c, true, false));
        define_ctx!(ret, "write", |e, c| SExp::do_print(e, c, false, true));
        define_ctx!(ret, "writeln", |e, c| SExp::do_print(e, c, true, true));

        // functional goodness
        define_ctx!(ret, "map", SExp::eval_map);
        define_ctx!(ret, "foldl", SExp::eval_fold);
        define_ctx!(ret, "filter", SExp::eval_filter);

        // Numerics
        define!(ret, "zero?", |e: SExp| Ok((e.car()? == 0.into()).into()));
        define_with!(ret, "add1", |e| e + 1., make_unary_numeric);
        define_with!(ret, "sub1", |e| e - 1., make_unary_numeric);
        define_with!(
            ret,
            "=",
            |l, r| (l - r).abs() < std::f64::EPSILON,
            make_binary_numeric
        );
        define_with!(ret, "<", |l, r| l < r, make_binary_numeric);
        define_with!(ret, ">", |l, r| l > r, make_binary_numeric);
        define_with!(ret, "abs", f64::abs, make_unary_numeric);
        ret.define("+", make_fold_numeric(0., std::ops::Add::add, Some("+")));
        define_with!(ret, "-", std::ops::Sub::sub, make_fold_from0_numeric);
        ret.define("*", make_fold_numeric(1., std::ops::Mul::mul, Some("*")));
        define_with!(ret, "/", std::ops::Div::div, make_fold_from0_numeric);
        define_with!(ret, "remainder", std::ops::Rem::rem, make_binary_numeric);
        define_with!(ret, "pow", f64::powf, make_binary_numeric);
        ret.define("pi", std::f64::consts::PI.into());

        // Vectors
        define_with!(
            ret,
            "make-vector",
            |e| match e {
                Atom(Number(n)) => Ok(Vector(vec![Null; n.floor() as usize])),
                _ => Err(Error::Type),
            },
            make_unary_expr
        );
        define_with!(
            ret,
            "vector-copy",
            |v| match v {
                Vector(vec) => Ok(Vector(vec.clone())),
                _ => Err(Error::Type),
            },
            make_unary_expr
        );
        define_with!(
            ret,
            "vector?",
            |e| match e {
                Vector(_) => Ok(true.into()),
                _ => Ok(false.into()),
            },
            make_unary_expr
        );
        define_with!(
            ret,
            "vector-length",
            |v| match v {
                Vector(vec) => Ok((vec.len() as f64).into()),
                _ => Err(Error::Type),
            },
            make_unary_expr
        );

        // Procedures
        define_with!(
            ret,
            "procedure?",
            |e| match e {
                Atom(Procedure { .. }) | Atom(CtxProcedure { .. }) => Ok(true.into()),
                _ => Ok(false.into()),
            },
            make_unary_expr
        );

        // Strings
        define!(ret, "string->list", |e| match e {
            Pair {
                head: box Atom(LispString(s)),
                tail: box Null,
            } => Ok(s.chars().map(SExp::from).collect()),
            _ => Err(Error::Type),
        });
        define!(ret, "list->string", |e| match e {
            Pair { .. } => {
                match e.into_iter().fold(Ok(String::new()), |s, e| match e {
                    Atom(Character(ref c)) => {
                        if let Ok(st) = s {
                            let mut stri = st;
                            stri.push(*c);
                            Ok(stri)
                        } else {
                            s
                        }
                    }
                    _ => Err(Error::Type),
                }) {
                    Ok(s) => Ok(Atom(LispString(s))),
                    Err(err) => Err(err),
                }
            }
            _ => Err(Error::Type),
        });

        ret
    }
}
