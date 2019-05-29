use std::fmt::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use super::super::Primitive::{
    Boolean, Character, Env, Number, Procedure, String as LispString, Symbol, Undefined, Void,
};
use super::super::SExp::{self, Atom, Null, Pair};
use super::super::{Error, Num, Result};

use super::super::proc::utils::*;
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
    ( $ctx:ident, $name:expr, $proc:expr, $arity:expr ) => {
        $ctx.lang.insert(
            $name.to_string(),
            $crate::SExp::from($crate::Proc::new(
                $crate::Func::Ctx(::std::rc::Rc::new($proc)),
                $arity,
                ::std::option::Option::Some($name),
            )),
        )
    };
}

macro_rules! define {
    ( $ctx:ident, $name:expr, $proc:expr, $arity:expr ) => {
        $ctx.lang.insert(
            $name.to_string(),
            $crate::SExp::from($crate::Proc::new(
                $crate::Func::Pure(::std::rc::Rc::new($proc)),
                $arity,
                Some($name),
            )),
        )
    };
}

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
        .replace("\\r", "\r")
        .replace("\\0", "\0")
        .replace("\\\"", "\"")
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
    /// assert_eq!(
    ///     ctx.eval(
    ///         sexp![SExp::sym("null?"), SExp::sym("null")]
    ///     ).unwrap(),
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
        define!(
            ret,
            "string->list",
            |e| match &e[0] {
                Atom(LispString(s)) => Ok(s.chars().map(SExp::from).collect()),
                exp => Err(Error::Type {
                    expected: "string",
                    given: exp.type_of().to_string()
                }),
            },
            3
        );
        define!(
            ret,
            "list->string",
            |e| match e {
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
            },
            1
        );

        ret
    }

    fn std(&mut self) {
        define!(self, "eq?", |e| Ok((e[0] == e[1]).into()), 2);
        define_with!(
            self,
            "eqv?",
            |e0, e1| Ok(match (e0, e1) {
                (Null, Null) => true,
                (Atom(Boolean(b0)), Atom(Boolean(b1))) => b0 == b1,
                (Atom(Character(c0)), Atom(Character(c1))) => c0 == c1,
                (Atom(Symbol(s0)), Atom(Symbol(s1))) => s0 == s1,
                (Atom(Number(n0)), Atom(Number(n1))) => n0 == n1,
                (Atom(Procedure(p0)), Atom(Procedure(p1))) => p0 == p1,
                _ => false,
            }
            .into()),
            make_binary_expr
        );
        define!(self, "equal?", |e| Ok((e[0] == e[1]).into()), 2);

        define!(self, "null?", |e| Ok((e == ((),).into()).into()), 1);
        self.lang.insert("null".to_string(), Null);
        define!(self, "void", |_| Ok(Atom(Void)), 0);
        define!(self, "list", Ok, (0,));
        define!(self, "not", |e| Ok((e == (false,).into()).into()), 1);

        define!(
            self,
            "cons",
            |e| {
                let (car, cdr) = e.split_car()?;
                let (car2, _) = cdr.split_car()?;
                Ok(car2.cons(car))
            },
            2
        );

        define_with!(self, "car", SExp::car, make_unary_expr);
        define_with!(self, "cdr", SExp::cdr, make_unary_expr);

        define_ctx!(
            self,
            "set-car!",
            |c, e| {
                let (car, cdr) = e.split_car()?;
                let new = cdr.car()?;

                match car {
                    Atom(Symbol(key)) => {
                        if let Some(mut val) = c.get(&key) {
                            val.set_car(c.eval(new)?)?;
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
            },
            2
        );

        define_ctx!(
            self,
            "set-cdr!",
            |c, e| {
                let (car, cdr) = e.split_car()?;
                let new = cdr.car()?;

                match car {
                    Atom(Symbol(key)) => {
                        if let Some(mut val) = c.get(&key) {
                            val.set_cdr(c.eval(new)?)?;
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
            },
            2
        );

        define_with!(
            self,
            "type-of",
            |e| Ok(SExp::from(e.type_of().to_string())),
            make_unary_expr
        );

        // i/o
        define_ctx!(
            self,
            "display",
            |e, c| Self::do_print(e, c, false, false),
            1
        );
        define_ctx!(
            self,
            "displayln",
            |e, c| Self::do_print(e, c, true, false),
            1
        );
        define_ctx!(self, "write", |e, c| Self::do_print(e, c, false, true), 1);
        define_ctx!(self, "writeln", |e, c| Self::do_print(e, c, true, true), 1);

        #[cfg(not(target_arch = "wasm32"))]
        define_ctx!(
            self,
            "require",
            |c, e| match c.eval(e.car()?)? {
                Atom(LispString(f_name)) => c.run(&fs::read_to_string(f_name)?),
                other => Err(Error::Type {
                    expected: "string",
                    given: other.type_of().to_string(),
                }),
            },
            1
        );

        // functional goodness
        define_ctx!(self, "map", Self::eval_map, 2);
        define_ctx!(self, "foldl", Self::eval_fold, 3);
        define_ctx!(self, "filter", Self::eval_filter, 2);

        // procedures
        define_with!(
            self,
            "procedure-arity",
            |e| match e {
                Atom(Procedure(p)) => Ok(p.get_arity()),
                other => Err(Error::Type {
                    expected: "procedure",
                    given: other.type_of().to_string(),
                }),
            },
            make_unary_expr
        );
        let check_proc_arity = |e0, e1| match (e0, e1) {
            (Atom(Procedure(p)), Atom(Number(n))) => Ok(p.check_arity(n.into()).is_ok().into()),
            (Atom(Procedure(_)), other) => Err(Error::Type {
                expected: "number",
                given: other.type_of().to_string(),
            }),
            (other, _) => Err(Error::Type {
                expected: "procedure",
                given: other.type_of().to_string(),
            }),
        };
        define_with!(
            self,
            "procedure-arity-valid?",
            check_proc_arity,
            make_binary_expr
        );
        define_with!(
            self,
            "procedure-of-arity?",
            move |e0, e1| if let Atom(Procedure(_)) = e0 {
                check_proc_arity(e0, e1)
            } else {
                Ok(false.into())
            },
            make_binary_expr
        );

        define_with!(
            self,
            "thunk?",
            |e| Ok(if let Atom(Procedure(p)) = e {
                p.thunk()
            } else {
                false
            }
            .into()),
            make_unary_expr
        );
    }

    fn do_print(&mut self, expr: SExp, newline: bool, debug: bool) -> Result {
        let ending = if newline { "\n" } else { "" };
        let hevl = self.eval(expr.car()?)?;
        let unescaped = unescape(&if debug {
            format!("{:?}{}", hevl, ending)
        } else {
            format!("{}{}", hevl, ending)
        });
        write!(self, "{}", unescaped)?;

        Ok(Atom(Undefined))
    }

    fn eval_map(&mut self, expr: SExp) -> Result {
        let (head, tail) = expr.split_car()?;
        self.eval(tail.car()?)?
            .into_iter()
            .map(|e| self.eval(Null.cons(e).cons(head.to_owned())))
            .collect()
    }

    fn eval_fold(&mut self, expr: SExp) -> Result {
        let (head, tail) = expr.split_car()?;
        let (init, tail) = tail.split_car()?;

        self.eval(tail.car()?)?
            .into_iter()
            .fold(Ok(init), |a, e| match a {
                Ok(acc) => self.eval(Null.cons(e).cons(acc).cons(head.to_owned())),
                err => err,
            })
    }

    fn eval_filter(&mut self, expr: SExp) -> Result {
        let (predicate, tail) = expr.split_car()?;

        self.eval(tail.car()?)?
            .into_iter()
            .filter_map(
                |e| match self.eval(Null.cons(e.clone()).cons(predicate.to_owned())) {
                    Ok(Atom(Boolean(false))) => None,
                    Ok(_) => Some(Ok(e)),
                    err => Some(err),
                },
            )
            .collect()
    }

    fn num_base(&mut self) {
        define!(
            self,
            "zero?",
            |e: SExp| Ok((e.car()? == 0.into()).into()),
            1
        );
        define_with!(self, "add1", |e| e + Num::Int(1), make_unary_numeric);
        define_with!(self, "sub1", |e| e - Num::Int(1), make_unary_numeric);

        define_with!(self, "=", |l, r| l == r, make_binary_numeric);

        define_with!(self, "<", |l, r| l < r, make_binary_numeric);
        define_with!(self, ">", |l, r| l > r, make_binary_numeric);
        define_with!(self, "abs", Num::abs, make_unary_numeric);

        self.lang.insert(
            "+".to_string(),
            make_fold_numeric(Num::Int(0), std::ops::Add::add, Some("+")),
        );

        define_with!(self, "-", std::ops::Sub::sub, make_fold_from0_numeric);

        self.lang.insert(
            "*".to_string(),
            make_fold_numeric(Num::Int(1), std::ops::Mul::mul, Some("*")),
        );

        define_with!(self, "/", std::ops::Div::div, make_fold_from0_numeric);
        define_with!(self, "remainder", std::ops::Rem::rem, make_binary_numeric);
        define_with!(self, "pow", Num::pow, make_binary_numeric);

        self.lang
            .insert("pi".to_string(), std::f64::consts::PI.into());
    }
}
