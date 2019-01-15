use super::super::Primitive::{
    Character, Env, Number, Procedure, String as LispString, Symbol, Void,
};
use super::super::SExp::{self, Atom, Null, Pair, Vector};
use super::super::{Env as Environment, Error};

use super::utils::*;
use super::Context;

mod tests;

macro_rules! define_with {
    ( $ctx:ident, $name:expr, $proc:expr, $tform:expr ) => {
        $ctx.lang
            .insert($name.to_string(), $tform($proc, Some($name)))
    };
}

macro_rules! define_ctx {
    ( $ctx:ident, $name:expr, $proc:expr ) => {
        define_with!($ctx, $name, $proc, $crate::SExp::ctx_proc)
    };
}

macro_rules! define {
    ( $ctx:ident, $name:expr, $proc:expr ) => {
        $ctx.lang
            .insert($name.to_string(), $crate::SExp::proc($proc, Some($name)))
    };
}

macro_rules! tup_ctx_env {
    ( $name:expr, $proc:expr ) => {
        (
            $name.to_string(),
            $crate::SExp::ctx_proc($proc, Some($name)),
        )
    };
}

impl Context {
    pub(super) fn core() -> Environment {
        [
            tup_ctx_env!("eval", |e, c| e.car()?.eval(c)?.eval(c)),
            tup_ctx_env!("apply", SExp::do_apply),
            tup_ctx_env!("and", SExp::eval_and),
            tup_ctx_env!("begin", SExp::eval_begin),
            tup_ctx_env!("case", SExp::eval_case),
            tup_ctx_env!("cond", SExp::eval_cond),
            tup_ctx_env!("define", SExp::eval_define),
            tup_ctx_env!("if", SExp::eval_if),
            tup_ctx_env!("lambda", |e, c| SExp::eval_lambda(e, c, false)),
            tup_ctx_env!("let", SExp::eval_let),
            tup_ctx_env!("named-lambda", |e, c| SExp::eval_lambda(e, c, true)),
            tup_ctx_env!("or", SExp::eval_or),
            tup_ctx_env!("quote", SExp::eval_quote),
            tup_ctx_env!("set!", SExp::eval_set),
        ]
        .iter()
        .cloned()
        .collect()
    }

    /// Base context - defines a number of useful functions and constants for
    /// use in the runtime.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = Context::base();
    ///
    /// assert_eq!(
    ///     sexp![SExp::sym("null?"), SExp::sym("null")].eval(&mut ctx).unwrap(),
    ///     SExp::from(true),
    /// );
    ///
    /// println!("{}", ctx.get("eq?").unwrap());   // "#<procedure>"
    /// println!("{}", ctx.get("+").unwrap());     // "#<procedure>"
    /// ```
    pub fn base() -> Self {
        let mut ret = Self::default();

        // The basics
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
        ret.lang.insert("null".to_string(), Null);
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
            } => Ok(elem2.cons(*elem1)),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        });
        define_with!(ret, "car", SExp::car, make_unary_expr);
        define_with!(ret, "cdr", SExp::cdr, make_unary_expr);
        define_ctx!(ret, "set-car!", |e, c| match e {
            Pair {
                head: box Atom(Symbol(s)),
                tail:
                    box Pair {
                        head: box new,
                        tail: box Null,
                    },
            } => {
                if let Some(mut val) = c.get(&s) {
                    let new_val = new.eval(c)?;
                    val.set_car(new_val)?;
                    c.set(&s, val)
                } else {
                    Err(Error::UndefinedSymbol { sym: s })
                }
            }
            _ => Err(Error::Type),
        });
        define_ctx!(ret, "set-cdr!", |e, c| match e {
            Pair {
                head: box Atom(Symbol(s)),
                tail:
                    box Pair {
                        head: box new,
                        tail: box Null,
                    },
            } => {
                if let Some(mut val) = c.get(&s) {
                    let new_val = new.eval(c)?;
                    val.set_cdr(new_val)?;
                    c.set(&s, val)
                } else {
                    Err(Error::UndefinedSymbol { sym: s })
                }
            }
            _ => Err(Error::Type),
        });
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
        ret.lang.insert(
            "+".to_string(),
            make_fold_numeric(0., std::ops::Add::add, Some("+")),
        );
        define_with!(ret, "-", std::ops::Sub::sub, make_fold_from0_numeric);
        ret.lang.insert(
            "*".to_string(),
            make_fold_numeric(1., std::ops::Mul::mul, Some("*")),
        );
        define_with!(ret, "/", std::ops::Div::div, make_fold_from0_numeric);
        define_with!(ret, "remainder", std::ops::Rem::rem, make_binary_numeric);
        define_with!(ret, "pow", f64::powf, make_binary_numeric);
        ret.lang
            .insert("pi".to_string(), std::f64::consts::PI.into());

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
                Atom(Procedure { .. }) => Ok(true.into()),
                _ => Ok(false.into()),
            },
            make_unary_expr
        );

        // Environments
        define_with!(
            ret,
            "environment?",
            |e| match e {
                Atom(Env(_)) => Ok(true.into()),
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
