use std::fmt;

/// Multipurpose error type.
#[derive(Debug)]
pub enum Error {
    Syntax {
        exp: String,
    },
    Type {
        expected: &'static str,
        given: String,
    },
    UndefinedSymbol {
        sym: String,
    },
    Arity {
        expected: usize,
        given: usize,
    },
    ArityMin {
        expected: usize,
        given: usize,
    },
    ArityMax {
        expected: usize,
        given: usize,
    },
    NotAList {
        atom: String,
    },
    NullList,
    NotAProcedure {
        exp: String,
    },
    Index {
        i: usize,
    },
    IO(::std::fmt::Error),
}

impl ::std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Syntax { exp } => write!(f, "Could not parse expression: {}", exp),
            Error::Type { expected, given } => {
                write!(f, "Type error: expected {}, got {}", expected, given)
            }
            Error::UndefinedSymbol { sym } => write!(f, "Undefined symbol: {}", sym),
            Error::Arity { expected, given } => write!(
                f,
                "Arity mismatch: expected {} parameters, got {}.",
                expected, given
            ),
            Error::ArityMin { expected, given } => write!(
                f,
                "Arity mismatch: expected at least {} parameters, got {}.",
                expected, given
            ),
            Error::ArityMax { expected, given } => write!(
                f,
                "Arity mismatch: expected at most {} parameters, got {}.",
                expected, given
            ),
            Error::NotAList { atom } => write!(f, "Expected a list, got {}", atom),
            Error::NullList => write!(f, "Expected a pair, got null."),
            Error::NotAProcedure { exp } => write!(f, "{} is not a procedure.", exp),
            Error::Index { i } => write!(f, "Tried to access invalid index: [{}]", i),
            Error::IO(err) => write!(f, "I/O error: {}", err),
        }
    }
}
