use super::super::super::proc::utils::*;
use super::super::super::Error;
use super::super::super::Primitive::{Number, Symbol, Undefined, Vector};
use super::super::super::SExp::{Atom, Null};
use super::super::Context;

macro_rules! define_with {
    ( $ctx:ident, $name:expr, $proc:expr, $tform:expr ) => {
        $ctx.lang
            .insert($name.to_string(), $tform($proc, Some($name)))
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

macro_rules! define_ctx {
    ( $ctx:ident, $name:expr, $proc:expr, $arity:expr ) => {
        $ctx.lang.insert(
            $name.to_string(),
            $crate::SExp::from($crate::Proc::new(
                $crate::Func::Ctx(::std::rc::Rc::new($proc)),
                $arity,
                Some($name),
            )),
        )
    };
}

impl Context {
    pub(super) fn vector(&mut self) {
        define!(self, "make-vector", |exp| {
            let (first_arg, rest) = exp.split_car()?;
            let second_arg = match rest {
                Null => Null,
                a @ Atom(_) => a,
                _ => rest.car()?,
            };

            match first_arg {
                Atom(Number(n)) => Ok(Atom(Vector(vec![second_arg; n.into()]))),
                _ => Err(Error::Type {
                    expected: "number",
                    given: first_arg.type_of().to_string(),
                }),
            }
        }, (1, 2));

        define_with!(
            self,
            "vector-copy",
            |v| match v {
                Atom(Vector(vec)) => Ok(Atom(Vector(vec.clone()))),
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
                Atom(Vector(_)) => Ok(true.into()),
                _ => Ok(false.into()),
            },
            make_unary_expr
        );

        define_with!(
            self,
            "vector-length",
            |v| match v {
                Atom(Vector(vec)) => Ok(vec.len().into()),
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
                (Atom(Vector(vec)), Atom(Number(n))) => vec
                    .get(usize::from(n))
                    .map(ToOwned::to_owned)
                    .ok_or(Error::Index { i: n.into() }),
                (Atom(Vector(_)), i) => Err(Error::Type {
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
            |ctx, expr| {
                let (s, tail) = expr.split_car()?;
                let (num, tail) = tail.split_car()?;
                let head = tail.car()?;

                let sym = match s {
                    Atom(Symbol(sym)) => sym,
                    e => {
                        return Err(Error::Type {
                            expected: "symbol",
                            given: e.type_of().to_string(),
                        });
                    }
                };
                let n = match ctx.eval(num)? {
                    Atom(Number(n)) => n,
                    e => {
                        return Err(Error::Type {
                            expected: "number",
                            given: e.type_of().to_string(),
                        });
                    }
                };

                match ctx.get(&sym) {
                    Some(Atom(Vector(mut vec))) => {
                        vec[usize::from(n)] = ctx.eval(head)?;
                        ctx.set(&sym, Atom(Vector(vec))).unwrap();
                        Ok(Atom(Undefined))
                    }
                    Some(val) => Err(Error::Type {
                        expected: "vector",
                        given: val.type_of().to_string(),
                    }),
                    None => Err(Error::UndefinedSymbol { sym }),
                }
            },
            3
        );

        define_ctx!(
            self,
            "vector-map",
            |ctx, expr| {
                let (proc, tail) = expr.split_car()?;

                let vec = match tail.car()? {
                    Atom(Vector(v)) => v,
                    e => {
                        return Err(Error::Type {
                            expected: "vector",
                            given: e.type_of().to_string(),
                        });
                    }
                };

                let mut new_vec = Vec::new();
                for expression in vec {
                    new_vec.push(ctx.eval(Null.cons(expression).cons(proc.clone()))?);
                }
                Ok(Atom(Vector(new_vec)))
            },
            2
        );

        define_with!(
            self,
            "subvector",
            |v, start, end| match (v, start, end) {
                (Atom(Vector(vec)), Atom(Number(n0)), Atom(Number(n1))) => {
                    let (i0, i1) = (n0.into(), n1.into());
                    if i0 >= vec.len() {
                        return Err(Error::Index { i: i0 });
                    }
                    if i1 >= vec.len() {
                        return Err(Error::Index { i: i1 });
                    }

                    Ok(Atom(Vector(vec[i0..i1].to_vec())))
                }
                (Atom(Vector(_)), Atom(Number(_)), end) => Err(Error::Type {
                    expected: "number",
                    given: end.type_of().to_string(),
                }),
                (Atom(Vector(_)), start, _) => Err(Error::Type {
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
                (Atom(Vector(vec)), Atom(Number(n1))) => {
                    let i1 = n1.into();
                    if i1 >= vec.len() {
                        return Err(Error::Index { i: i1 });
                    }

                    Ok(Atom(Vector(vec[..i1].to_vec())))
                }
                (Atom(Vector(_)), end) => Err(Error::Type {
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
                (Atom(Vector(vec)), Atom(Number(n0))) => {
                    let i0 = n0.into();
                    if i0 >= vec.len() {
                        return Err(Error::Index { i: i0 });
                    }

                    Ok(Atom(Vector(vec[i0..].to_vec())))
                }
                (Atom(Vector(_)), start) => Err(Error::Type {
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
