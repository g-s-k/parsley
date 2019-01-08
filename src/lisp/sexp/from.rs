use super::super::Primitive;
use super::SExp::{self, Atom, Null, Pair};

#[macro_export]
macro_rules! sexp {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();

            $(
                temp_vec.push(SExp::from($x));
            )*

            SExp::from(temp_vec)
        }
    };
}

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
            head: Box::new(Self::from(v)),
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
            head: Box::new(Self::from(v1)),
            tail: Box::new(Self::from(v2)),
        }
    }
}

impl<T> From<&[T]> for SExp
where
    T: Into<SExp> + Clone,
{
    fn from(ary: &[T]) -> Self {
        ary.iter().rev().cloned().map(T::into).collect()
    }
}

impl<T> From<Vec<T>> for SExp
where
    T: Into<SExp>,
{
    fn from(ary: Vec<T>) -> Self {
        ary.into_iter().map(T::into).collect()
    }
}
