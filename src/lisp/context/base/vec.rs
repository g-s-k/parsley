use super::super::super::Error;
use super::super::super::Primitive::{Number, Symbol, Undefined};
use super::super::super::SExp::{Atom, Null, Pair, Vector};
use super::super::utils::*;
use super::super::Context;

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

impl Context {
    pub(super) fn vector(&mut self) {
        define_with!(
            self,
            "make-vector",
            |e| match e {
                Atom(Number(n)) => Ok(Vector(vec![Null; n.floor() as usize])),
                _ => Err(Error::Type),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector-copy",
            |v| match v {
                Vector(vec) => Ok(Vector(vec.clone())),
                _ => Err(Error::Type),
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
                _ => Err(Error::Type),
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
                _ => Err(Error::Type),
            },
            make_binary_expr
        );

        define_ctx!(self, "vector-set!", |expr, ctx| if let Pair {
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
                Some(_) => Err(Error::Type),
                None => Err(Error::UndefinedSymbol { sym }),
            }
        } else {
            Err(Error::Type)
        });

        define_ctx!(self, "vector-map", |expr, ctx| if let Pair {
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
                new_vec.push(Null.cons(expression).cons(proc.clone()).eval(ctx)?);
            }
            Ok(Vector(new_vec))
        } else {
            Err(Error::Type)
        });
    }
}
