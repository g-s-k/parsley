mod as_atom;
mod context;
mod errors;
mod primitives;
mod sexp;
mod tests;
mod utils;

pub use self::context::Context;
use self::errors::LispError;
use self::primitives::Primitive;
use self::sexp::SExp;

const NULL: SExp = SExp::List(Vec::new());

type LispResult = Result<SExp, LispError>;

pub fn parse(s: &str) -> LispResult {
    s.parse::<SExp>()
}
