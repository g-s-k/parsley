use super::super::LispError;
use super::super::Primitive::{Character, Number, String as LispString};
use super::super::SExp::{self, Atom, Null, Pair};

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
            SExp::from(|e| match e {
                Pair {
                    head: elem1,
                    tail:
                        box Pair {
                            head: elem2,
                            tail: box Null,
                        },
                } => Ok(SExp::from(elem1 == elem2)),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }),
        );
        ret.define(
            "null?",
            SExp::from(|e| {
                trace!("{}", e);
                Ok(SExp::from(match e {
                    Pair {
                        head: box Null,
                        tail: box Null,
                    } => true,
                    _ => false,
                }))
            }),
        );
        ret.define("null", Null.cons(Null).cons(SExp::make_symbol("quote")));
        ret.define(
            "cons",
            SExp::from(|e| match e {
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
            }),
        );
        ret.define(
            "car",
            SExp::from(|e| match e {
                Pair { head, .. } => head.car(),
                _ => Err(LispError::TypeError),
            }),
        );
        ret.define(
            "cdr",
            SExp::from(|e| match e {
                Pair { head, .. } => head.cdr(),
                _ => Err(LispError::TypeError),
            }),
        );

        // Numerics
        ret.define(
            "=",
            SExp::from(|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok(SExp::from((n1 - n2).abs() < std::f64::EPSILON)),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }),
        );
        ret.define(
            "<",
            SExp::from(|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok(SExp::from(n1 < n2)),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }),
        );
        ret.define(
            ">",
            SExp::from(|e| match e {
                Pair {
                    head: box Atom(Number(n1)),
                    tail:
                        box Pair {
                            head: box Atom(Number(n2)),
                            tail: box Null,
                        },
                } => Ok(SExp::from(n1 > n2)),
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            }),
        );
        ret.define(
            "+",
            SExp::from(|v: SExp| {
                v.into_iter().fold(Ok(SExp::from(0)), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n + na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }),
        );
        ret.define(
            "-",
            SExp::from(|e| match e {
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
            }),
        );
        ret.define(
            "*",
            SExp::from(|v: SExp| {
                v.into_iter().fold(Ok(SExp::from(1)), |a, e| match e {
                    Atom(Number(n)) => {
                        if let Ok(Atom(Number(na))) = a {
                            Ok(Atom(Number(n * na)))
                        } else {
                            a
                        }
                    }
                    _ => Err(LispError::TypeError),
                })
            }),
        );
        ret.define(
            "/",
            SExp::from(|e| match e {
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
            }),
        );

        // Strings
        ret.define(
            "string->list",
            SExp::from(|e| match e {
                Pair {
                    head: box Atom(LispString(s)),
                    tail: box Null,
                } => Ok(s.chars().map(|c| Atom(Character(c))).collect()),
                _ => Err(LispError::TypeError),
            }),
        );
        ret.define(
            "list->string",
            SExp::from(|e| match e {
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
            }),
        );

        ret
    }
}
