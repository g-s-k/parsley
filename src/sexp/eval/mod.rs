mod tests;

/// Evaluate one or more S-Expressions, in the base context or your own custom one.
///
/// # Examples
/// ```
/// use parsley::prelude::*;
/// assert!(eval!(sexp!["potato", true, 5]).is_ok());
/// assert!(eval!(sexp![SExp::sym("potato"), true, 5]).is_err());
/// ```
/// ```
/// use parsley::prelude::*;
///
/// let evaluated = eval!(
///     sexp![
///         SExp::sym("define"),
///         sexp![SExp::sym("square"), SExp::sym("x")],
///         sexp![SExp::sym("*"), SExp::sym("x"), SExp::sym("x")]
///     ],
///     sexp![SExp::sym("square"), 12]
/// );
///
/// assert!(evaluated.is_ok());
/// assert_eq!(evaluated.unwrap(), SExp::from(144));
/// ```
/// ```
/// use parsley::prelude::*;
/// let mut ctx = Context::base();
///
/// eval!(
///     ctx;
///     sexp![
///         SExp::sym("define"),
///         sexp![SExp::sym("potato"), SExp::sym("x"), SExp::sym("y")],
///         SExp::sym("y")
///     ]
/// );
/// let evaluated = eval!(ctx; sexp![SExp::sym("potato"), true, 5]);
///
/// assert!(evaluated.is_ok());
/// assert_eq!(evaluated.unwrap(), SExp::from(5));
/// ```
#[macro_export]
macro_rules! eval {
    ( $( $expression:expr ),* ) => {
        $crate::eval!($crate::Context::base(); $( $expression ),*)
    };
    ( $context:expr; $expression:expr ) => {
        $context.eval($expression)
    };
    ( $context:expr; $( $expression:expr ),* ) => {
        $context.eval(
            $crate::sexp![
                $crate::SExp::sym("begin"),
                $( $expression ),*
            ]
        )
    };
}
