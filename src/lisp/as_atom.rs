use super::{Primitive, SExp};

/// A straightforward way to get each kind of primitive.
pub trait AsAtom {
    fn as_atom(&self) -> SExp;
}

impl AsAtom for bool {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Boolean(*self))
    }
}

impl AsAtom for char {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Character(*self))
    }
}

impl AsAtom for f64 {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Number(*self))
    }
}

impl AsAtom for str {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::String(self.to_string()))
    }
}
