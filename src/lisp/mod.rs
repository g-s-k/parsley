mod as_atom;
mod context;
mod errors;
mod primitives;
mod sexp;
mod tests;
mod utils;

#[allow(unused_imports)]
use self::as_atom::AsAtom;
pub use self::context::Context;
pub use self::errors::LispError;
use self::primitives::Primitive;
pub use self::sexp::SExp;

const NULL: SExp = SExp::List(Vec::new());

type LispResult = Result<SExp, LispError>;
