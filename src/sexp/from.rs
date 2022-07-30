use super::super::Primitive;
use super::SExp::{self, Atom, Null, Pair};

/// Construct an S-Expression from a list of expressions.
///
/// # Example
/// ```
/// use parsley::{sexp, SExp};
///
/// assert_eq!(
///     sexp![5, "potato", true],
///     SExp::from((5, ("potato", (true, ()))))
/// );
/// ```
#[macro_export]
macro_rules! sexp {
    ( $( $e:expr ),* ) => {{
        $crate::SExp::from(&[ $( $crate::SExp::from($e) ),* ][..])
    }};
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
    T: Into<SExp>,
    U: Into<SExp>,
{
    fn from((v1, v2): (T, U)) -> Self {
        Pair {
            head: Box::new(v1.into()),
            tail: Box::new(v2.into()),
        }
    }
}

impl<T> From<&[T]> for SExp
where
    T: Into<SExp> + Clone,
{
    fn from(ary: &[T]) -> Self {
        ary.iter().cloned().map(T::into).collect()
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
