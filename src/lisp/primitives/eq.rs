use super::Primitive;

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Primitive::Void => {
                if let Primitive::Void = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Undefined => {
                if let Primitive::Undefined = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Boolean(b1) => {
                if let Primitive::Boolean(b2) = other {
                    b1 == b2
                } else {
                    false
                }
            }
            Primitive::Character(c1) => {
                if let Primitive::Character(c2) = other {
                    c1 == c2
                } else {
                    false
                }
            }
            Primitive::Number(n1) => {
                if let Primitive::Number(n2) = other {
                    n1 == n2
                } else {
                    false
                }
            }
            Primitive::String(s1) => {
                if let Primitive::String(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Symbol(s1) => {
                if let Primitive::Symbol(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Procedure(_) => false,
        }
    }
}
