use std::fmt;

use super::SExp;

#[derive(Debug)]
pub enum SyntaxError {
    UnmatchedQuote(String),
    UnmatchedParen {
        exp: String,
        expected: char,
        given: Option<char>,
    },
    InvalidCond(SExp),
    NotANumber(String),
    NotAPrimitive(String),
    NotAToken(String),
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedQuote(s) => write!(f, "Unmatched quote: {}", s),
            SyntaxError::UnmatchedParen {
                exp,
                expected,
                given: Some(g),
            } => write!(
                f,
                "Paren mismatch: expected {}, given {} in expression {}",
                expected, g, exp
            ),
            SyntaxError::UnmatchedParen { exp, expected, .. } => write!(
                f,
                "Paren mismatch: expected {} and no match found in expression {}",
                expected, exp
            ),
            SyntaxError::InvalidCond(e) => write!(f, "Invalid `cond` clause: {}", e),
            SyntaxError::NotANumber(s) => write!(f, "Could not parse as a number: {}", s),
            SyntaxError::NotAPrimitive(s) => {
                write!(f, "Could not parse as a primitive value: {}", s)
            }
            SyntaxError::NotAToken(s) => write!(f, "Unrecognized token: {}", s),
        }
    }
}

/// Multipurpose error type.
#[derive(Debug)]
pub enum Error {
    Syntax(SyntaxError),
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
    IO(String),
}

impl ::std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Syntax(err) => write!(f, "{}", err),
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

impl From<SyntaxError> for Error {
    fn from(e: SyntaxError) -> Self {
        Error::Syntax(e)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::IO(format!("{}", e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(format!("{}", e))
    }
}
