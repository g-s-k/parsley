pub mod context;
mod errors;
mod primitives;
mod sexp;
mod utils;

pub use self::context::Context;
pub use self::errors::Error;
use self::primitives::Primitive;
pub use self::sexp::SExp;

/// A shorthand Result type.
pub type Result = ::std::result::Result<SExp, Error>;
