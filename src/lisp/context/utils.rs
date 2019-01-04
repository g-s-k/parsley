use super::super::{LispError, Primitive, SExp};

pub fn make_unary_numeric<T>(f: impl Fn(f64) -> T + 'static) -> SExp
where
    T: Into<SExp>,
{
    (move |e| match e {
        SExp::Pair {
            head: box SExp::Atom(Primitive::Number(n)),
            ..
        } => Ok((f(n)).into()),
        _ => Err(LispError::TypeError),
    })
    .into()
}

pub fn make_binary_numeric<T>(f: impl Fn(f64, f64) -> T + 'static) -> SExp
where
    T: Into<SExp>,
{
    (move |e| match e {
        SExp::Pair {
            head: box SExp::Atom(Primitive::Number(n1)),
            tail:
                box SExp::Pair {
                    head: box SExp::Atom(Primitive::Number(n2)),
                    ..
                },
        } => Ok((f(n1, n2)).into()),
        _ => Err(LispError::TypeError),
    })
    .into()
}
