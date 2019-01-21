// there are a number of f64 <-> usize casts in this module, and clippy
// (understandably) isn't a big fan.
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

use super::super::super::Primitive::{Number, Symbol, Undefined};
use super::super::super::SExp::{Atom, Null, Pair, Vector};
use super::super::super::{Arity, Error};
use super::super::utils::*;
use super::super::Context;

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
                None,
                Some($name),
            )),
        )
    };
}

impl Context {
    pub(super) fn vector(&mut self) {
        define_with!(
            self,
            "make-vector",
            |e| match e {
                Atom(Number(n)) => Ok(Vector(vec![Null; n.floor() as usize])),
                _ => Err(Error::Type {
                    expected: "number",
                    given: e.type_of().to_string(),
                }),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector-copy",
            |v| match v {
                Vector(vec) => Ok(Vector(vec.clone())),
                _ => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector?",
            |e| match e {
                Vector(_) => Ok(true.into()),
                _ => Ok(false.into()),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector-length",
            |v| match v {
                Vector(vec) => Ok((vec.len() as f64).into()),
                _ => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector-ref",
            |v, i| match (v, i) {
                (Vector(vec), Atom(Number(n))) => vec
                    .get(n as usize)
                    .map(ToOwned::to_owned)
                    .ok_or(Error::Index { i: n as usize }),
                (Vector(_), i) => Err(Error::Type {
                    expected: "number",
                    given: i.type_of().to_string(),
                }),
                (v, _) => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_binary_expr
        );

        define_ctx!(
            self,
            "vector-set!",
            |ctx, expr| if let Pair {
                head: box Atom(Symbol(sym)),
                tail:
                    box Pair {
                        head: box Atom(Number(n)),
                        tail:
                            box Pair {
                                head,
                                tail: box Null,
                            },
                    },
            } = expr
            {
                match ctx.get(&sym) {
                    Some(Vector(mut vec)) => {
                        vec[n as usize] = *head;
                        ctx.set(&sym, Vector(vec)).unwrap();
                        Ok(Atom(Undefined))
                    }
                    Some(val) => Err(Error::Type {
                        expected: "vector",
                        given: val.type_of().to_string(),
                    }),
                    None => Err(Error::UndefinedSymbol { sym }),
                }
            } else {
                Err(Error::Type {
                    expected: "vector",
                    given: expr.type_of().to_string(),
                })
            },
            Arity::Exact(2)
        );

        define_ctx!(
            self,
            "vector-map",
            |ctx, expr| if let Pair {
                head: box proc,
                tail:
                    box Pair {
                        head: box Vector(vec),
                        tail: box Null,
                    },
            } = expr
            {
                let mut new_vec = Vec::new();
                for expression in vec {
                    new_vec.push(ctx.eval(Null.cons(expression).cons(proc.clone()))?);
                }
                Ok(Vector(new_vec))
            } else {
                Err(Error::Type {
                    expected: "procedure",
                    given: expr.type_of().to_string(),
                })
            },
            Arity::Exact(2)
        );

        define_with!(
            self,
            "subvector",
            |v, start, end| match (v, start, end) {
                (Vector(vec), Atom(Number(n0)), Atom(Number(n1))) => {
                    let (i0, i1) = (n0 as usize, n1 as usize);
                    if i0 >= vec.len() {
                        return Err(Error::Index { i: i0 });
                    }
                    if i1 >= vec.len() {
                        return Err(Error::Index { i: i1 });
                    }

                    Ok(Vector(vec[i0..i1].to_vec()))
                }
                (Vector(_), Atom(Number(_)), end) => Err(Error::Type {
                    expected: "number",
                    given: end.type_of().to_string(),
                }),
                (Vector(_), start, _) => Err(Error::Type {
                    expected: "number",
                    given: start.type_of().to_string(),
                }),
                (v, _, _) => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_ternary_expr
        );

        define_with!(
            self,
            "vector-head",
            |v, end| match (v, end) {
                (Vector(vec), Atom(Number(n1))) => {
                    let i1 = n1 as usize;
                    if i1 >= vec.len() {
                        return Err(Error::Index { i: i1 });
                    }

                    Ok(Vector(vec[..i1].to_vec()))
                }
                (Vector(_), end) => Err(Error::Type {
                    expected: "number",
                    given: end.type_of().to_string(),
                }),
                (v, _) => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_binary_expr
        );

        define_with!(
            self,
            "vector-tail",
            |v, start| match (v, start) {
                (Vector(vec), Atom(Number(n0))) => {
                    let i0 = n0 as usize;
                    if i0 >= vec.len() {
                        return Err(Error::Index { i: i0 });
                    }

                    Ok(Vector(vec[i0..].to_vec()))
                }
                (Vector(_), start) => Err(Error::Type {
                    expected: "number",
                    given: start.type_of().to_string(),
                }),
                (v, _) => Err(Error::Type {
                    expected: "vector",
                    given: v.type_of().to_string(),
                }),
            },
            make_binary_expr
        );
    }
}
