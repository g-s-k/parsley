/// Multipurpose error type.
#[derive(Debug, Fail)]
pub enum LispError {
    #[fail(display = "could not parse expression: {}", exp)]
    SyntaxError { exp: String },
    #[fail(display = "type error")]
    TypeError,
    #[fail(display = "symbol is not defined: {}", sym)]
    UndefinedSymbol { sym: String },
    #[fail(
        display = "arity mismatch: expected {} params, given {}.",
        expected, given
    )]
    ArityMismatch { expected: usize, given: usize },
    #[fail(display = "{} expects at least one argument.", symbol)]
    NoArgumentsProvided { symbol: String },
    #[fail(display = "Expected a list, got {}.", atom)]
    NotAList { atom: String },
    #[fail(display = "Expected a pair, got the null list.")]
    NullList,
    #[fail(display = "{} is not a procedure.", exp)]
    NotAProcedure { exp: String },
}
