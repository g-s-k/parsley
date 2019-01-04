use super::super::LispError;
use super::super::Primitive::{Character, Number, String as LispString};
use super::super::SExp::{self, Atom, Null, Pair};

use super::utils::*;
use super::Context;

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
    ///     SExp::from((null_fn, (null_const,))).eval(&mut ctx).unwrap(),
    ///     SExp::from(true),
    /// );
    ///
    /// println!("{}", ctx.get("eq?").unwrap());   // "#<procedure>"
    /// println!("{}", ctx.get("+").unwrap());     // "#<procedure>"
    /// ```
    pub fn base() -> Self {
        let mut ret = Self::default();

        // The basics
        ret.define(
            "eq?",
            (|e| match e {
                Pair {
                    head: elem1,
                    tail:
                        box Pair {
                            head: elem2,
                            tail: box Null,
                        },
                } => Ok((elem1 == elem2).into()),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            })
            .into(),
        );
        ret.define("null?", (|e| Ok((e == ((),).into()).into())).into());
        ret.define("null", (SExp::make_symbol("quote"), ((),)).into());
        ret.define("not", (|e| Ok((e == (false,).into()).into())).into());
        ret.define(
            "cons",
            (|e| match e {
                Pair {
                    head: elem1,
                    tail:
                        box Pair {
                            head: elem2,
                            tail: box Null,
                        },
                } => Ok(Null
                    .cons(elem2.cons(*elem1))
                    .cons(SExp::make_symbol("quote"))),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            })
            .into(),
        );
        ret.define(
            "car",
            (|e| match e {
                Pair { head, .. } => head.car(),
                _ => Err(LispError::TypeError),
            })
            .into(),
        );
        ret.define(
            "cdr",
            (|e| match e {
                Pair { head, .. } => head.cdr(),
                _ => Err(LispError::TypeError),
            })
            .into(),
        );
        ret.define(
            "type-of",
            (|e| match e {
                Pair { head, .. } => Ok(head.type_of().into()),
                _ => Err(LispError::TypeError),
            })
            .into(),
        );

        // Numerics
        ret.define(
            "=",
            (|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok(((n1 - n2).abs() < std::f64::EPSILON).into()),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            })
            .into(),
        );
        ret.define(
            "<",
            (|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok((n1 < n2).into()),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            })
            .into(),
        );
        ret.define(
            ">",
            (|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok((n1 > n2).into()),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            })
            .into(),
        );
        ret.define("abs", make_unary_numeric(f64::abs));
        ret.define(
            "+",
            (|v: SExp| {
                v.into_iter().fold(Ok(0.into()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n + na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            })
            .into(),
        );
        ret.define(
            "-",
            (|e| match e {
                Null => Err(LispError::TypeError),
                a @ Atom(_) => Err(LispError::NotAList {
                    atom: a.to_string(),
                }),
                Pair {
                    head: box Atom(Number(n)),
                    tail,
                } => {
                    let mut state = n;

                    for exp in tail.into_iter() {
                        match exp {
                            Atom(Number(n2)) => state -= n2,
                            _ => return Err(LispError::TypeError),
                        }
                    }

                    Ok(Atom(Number(state)))
                }
                _ => Err(LispError::TypeError),
            })
            .into(),
        );
        ret.define(
            "*",
            (|v: SExp| {
                v.into_iter().fold(Ok(1.into()), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n * na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            })
            .into(),
        );
        ret.define(
            "/",
            (|e| match e {
                Null => Err(LispError::TypeError),
                a @ Atom(_) => Err(LispError::NotAList {
                    atom: a.to_string(),
                }),
                Pair {
                    head: box Atom(Number(n)),
                    tail,
                } => {
                    let mut state = n;

                    for exp in tail.into_iter() {
                        match exp {
                            Atom(Number(n2)) => state /= n2,
                            _ => return Err(LispError::TypeError),
                        }
                    }

                    Ok(Atom(Number(state)))
                }
                _ => Err(LispError::TypeError),
            })
            .into(),
        );
        ret.define("pow", make_binary_numeric(f64::powf));
        ret.define("pi", std::f64::consts::PI.into());

        // Strings
        ret.define(
            "string->list",
            (|e| match e {
                Pair {
                    head: box Atom(LispString(s)),
                    tail: box Null,
                } => Ok(s.chars().map(|c| Atom(Character(c))).collect()),
                _ => Err(LispError::TypeError),
            })
            .into(),
        );
        ret.define(
            "list->string",
            (|e| match e {
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
                        _ => Err(LispError::TypeError),
                    }) {
                        Ok(s) => Ok(Atom(LispString(s))),
                        Err(err) => Err(err),
                    }
                }
                _ => Err(LispError::TypeError),
            })
            .into(),
        );

        ret
    }
}
