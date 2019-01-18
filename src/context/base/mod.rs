use super::super::Primitive::{Character, Env, Procedure, String as LispString, Symbol, Void};
use super::super::SExp::{self, Atom, Null, Pair};
use super::super::{Env as Environment, Error};

use super::utils::*;
use super::Context;

mod tests;
mod vec;

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
        ret.std();
        ret.num_base();
        ret.vector();

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
        define!(ret, "string->list", |e| match e.len() {
            1 => match &e[0] {
                Atom(LispString(s)) => Ok(s.chars().map(SExp::from).collect()),
                exp => Err(Error::Type {
                    expected: "string",
                    given: exp.type_of().to_string()
                }),
            },
            given => Err(Error::Arity { expected: 1, given }),
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
                    _ => Err(Error::Type {
                        expected: "char",
                        given: e.type_of().to_string(),
                    }),
                }) {
                    Ok(s) => Ok(Atom(LispString(s))),
                    Err(err) => Err(err),
                }
            }
            _ => Err(Error::Type {
                expected: "list",
                given: e.type_of().to_string()
            }),
        });

        ret
    }

    fn std(&mut self) {
        define!(self, "eq?", |e| match e.len() {
            2 => Ok((e[0] == e[1]).into()),
            given => Err(Error::Arity { expected: 2, given }),
        });

        define!(self, "null?", |e| Ok((e == ((),).into()).into()));
        self.lang.insert("null".to_string(), Null);
        define!(self, "void", |_| Ok(Atom(Void)));
        define!(self, "list", Ok);
        define!(self, "not", |e| Ok((e == (false,).into()).into()));

        define!(self, "cons", |e| match e.len() {
            2 => {
                let (car, cdr) = e.split_car()?;
                let (car2, _) = cdr.split_car()?;
                Ok(car2.cons(car))
            }
            given => Err(Error::Arity { expected: 2, given }),
        });

        define_with!(self, "car", SExp::car, make_unary_expr);
        define_with!(self, "cdr", SExp::cdr, make_unary_expr);

        define_ctx!(self, "set-car!", |e, c| match e.len() {
            2 => {
                let (car, cdr) = e.split_car()?;
                let new = cdr.car()?;

                match car {
                    Atom(Symbol(key)) => {
                        if let Some(mut val) = c.get(&key) {
                            let new_val = new.eval(c)?;
                            val.set_car(new_val)?;
                            c.set(&key, val)
                        } else {
                            Err(Error::UndefinedSymbol { sym: key })
                        }
                    }
                    other => Err(Error::Type {
                        expected: "symbol",
                        given: other.type_of().to_string(),
                    }),
                }
            }
            given => Err(Error::Arity { expected: 2, given }),
        });

        define_ctx!(self, "set-cdr!", |e, c| match e.len() {
            2 => {
                let (car, cdr) = e.split_car()?;
                let new = cdr.car()?;

                match car {
                    Atom(Symbol(key)) => {
                        if let Some(mut val) = c.get(&key) {
                            let new_val = new.eval(c)?;
                            val.set_cdr(new_val)?;
                            c.set(&key, val)
                        } else {
                            Err(Error::UndefinedSymbol { sym: key })
                        }
                    }
                    other => Err(Error::Type {
                        expected: "symbol",
                        given: other.type_of().to_string(),
                    }),
                }
            }
            given => Err(Error::Arity { expected: 2, given }),
        });

        define_with!(
            self,
            "type-of",
            |e| Ok(SExp::from(e.type_of().to_string())),
            make_unary_expr
        );

        // i/o
        define_ctx!(self, "display", |e, c| SExp::do_print(e, c, false, false));
        define_ctx!(self, "displayln", |e, c| SExp::do_print(e, c, true, false));
        define_ctx!(self, "write", |e, c| SExp::do_print(e, c, false, true));
        define_ctx!(self, "writeln", |e, c| SExp::do_print(e, c, true, true));

        // functional goodness
        define_ctx!(self, "map", SExp::eval_map);
        define_ctx!(self, "foldl", SExp::eval_fold);
        define_ctx!(self, "filter", SExp::eval_filter);
    }

    fn num_base(&mut self) {
        define!(self, "zero?", |e: SExp| Ok((e.car()? == 0.into()).into()));
        define_with!(self, "add1", |e| e + 1., make_unary_numeric);
        define_with!(self, "sub1", |e| e - 1., make_unary_numeric);

        define_with!(
            self,
            "=",
            |l, r| (l - r).abs() < std::f64::EPSILON,
            make_binary_numeric
        );

        define_with!(self, "<", |l, r| l < r, make_binary_numeric);
        define_with!(self, ">", |l, r| l > r, make_binary_numeric);
        define_with!(self, "abs", f64::abs, make_unary_numeric);

        self.lang.insert(
            "+".to_string(),
            make_fold_numeric(0., std::ops::Add::add, Some("+")),
        );

        define_with!(self, "-", std::ops::Sub::sub, make_fold_from0_numeric);

        self.lang.insert(
            "*".to_string(),
            make_fold_numeric(1., std::ops::Mul::mul, Some("*")),
        );

        define_with!(self, "/", std::ops::Div::div, make_fold_from0_numeric);
        define_with!(self, "remainder", std::ops::Rem::rem, make_binary_numeric);
        define_with!(self, "pow", f64::powf, make_binary_numeric);

        self.lang
            .insert("pi".to_string(), std::f64::consts::PI.into());
    }
}
