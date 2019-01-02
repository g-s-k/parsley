mod context;
mod errors;
mod primitives;
mod sexp;
mod tests;
mod utils;

pub use self::context::Context;
pub use self::errors::LispError;
use self::primitives::Primitive;
pub use self::sexp::SExp;

/// A shorthand Result type.
pub type LispResult = Result<SExp, LispError>;
