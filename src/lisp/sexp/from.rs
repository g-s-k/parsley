use super::super::Primitive;
use super::SExp::{self, *};

impl<T> From<T> for SExp
where
    Primitive: From<T>,
{
    fn from(val: T) -> Self {
        Atom(Primitive::from(val))
    }
}

impl From<()> for SExp {
    fn from(_: ()) -> Self {
        Null
    }
}

impl<T> From<(T,)> for SExp
where
    SExp: From<T>,
{
    fn from((v,): (T,)) -> Self {
        Pair {
            head: Box::new(SExp::from(v)),
            tail: Box::new(Null),
        }
    }
}

impl<T, U> From<(T, U)> for SExp
where
    SExp: From<T> + From<U>,
{
    fn from((v1, v2): (T, U)) -> Self {
        Pair {
            head: Box::new(SExp::from(v1)),
            tail: Box::new(SExp::from(v2)),
        }
    }
}
