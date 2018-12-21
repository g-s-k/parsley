mod as_atom;
mod context;
mod errors;
mod primitives;
mod sexp;
mod tests;
mod utils;

pub use self::as_atom::AsAtom;
pub use self::context::Context;
pub use self::errors::LispError;
use self::primitives::Primitive;
pub use self::sexp::SExp;

/// The null list.
pub const NULL: SExp = SExp::List(Vec::new());

/// A shorthand Result type.
pub type LispResult = Result<SExp, LispError>;

/// Parse and validate LISP source.
///
/// # Example
/// ```
/// use parsley::{NULL, parse};
/// assert_eq!(parse("()").unwrap(), NULL);
/// ```
pub fn parse(s: &str) -> LispResult {
    s.parse::<SExp>()
}
