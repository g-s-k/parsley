use super::super::proc::utils::*;
use super::super::Num;
use super::Context;

macro_rules! define_with {
    ( $ctx:ident, $name:expr, $proc:expr, $tform:expr ) => {
        $ctx.define($name, $tform($proc, Some($name)))
    };
}

impl Context {
    /// Math functions that are less commonly used. Intended to be layered on top of the base context.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = &mut Context::base().math();
    /// let mut asrt = |lhs, rhs| {
    ///     assert_eq!(ctx.run(lhs).unwrap(), ctx.run(rhs).unwrap())
    /// };
    ///
    /// asrt("(is-nan NaN)", "#t");
    /// asrt("(floor -4.07326)", "-5");
    /// asrt("(ceil 7.1)", "8");
    /// asrt("(hypot 3 4)", "5");
    /// asrt("(recip 100)", "0.01");
    /// asrt("(log (exp 7))", "7");
    /// ```
    pub fn math(mut self) -> Self {
        // identification
        define_with!(self, "is-nan", Num::is_nan, make_unary_numeric);
        define_with!(self, "is-infinite", Num::is_infinite, make_unary_numeric);
        define_with!(self, "is-finite", Num::is_finite, make_unary_numeric);
        define_with!(
            self,
            "is-positive",
            Num::is_sign_positive,
            make_unary_numeric
        );
        define_with!(
            self,
            "is-negative",
            Num::is_sign_negative,
            make_unary_numeric
        );

        // rounding etc.
        define_with!(self, "floor", Num::floor, make_unary_numeric);
        define_with!(self, "ceil", Num::ceil, make_unary_numeric);
        define_with!(self, "round", Num::round, make_unary_numeric);
        define_with!(self, "trunc", Num::trunc, make_unary_numeric);
        define_with!(self, "fract", Num::fract, make_unary_numeric);
        define_with!(self, "sign", Num::signum, make_unary_numeric);

        // exponents, roots, and logs
        define_with!(self, "recip", Num::recip, make_unary_numeric);
        define_with!(self, "sqrt", Num::sqrt, make_unary_numeric);
        define_with!(self, "cube-root", Num::cbrt, make_unary_numeric);
        define_with!(self, "exp", Num::exp, make_unary_numeric);
        define_with!(self, "log", Num::ln, make_unary_numeric);
        define_with!(self, "exp-2", Num::exp2, make_unary_numeric);
        define_with!(self, "log-2", Num::log2, make_unary_numeric);
        define_with!(self, "log-10", Num::log10, make_unary_numeric);
        define_with!(self, "log-n", Num::log, make_binary_numeric);

        // trigonometry
        define_with!(self, "hypot", Num::hypot, make_binary_numeric);
        define_with!(self, "sin", Num::sin, make_unary_numeric);
        define_with!(self, "cos", Num::cos, make_unary_numeric);
        define_with!(self, "tan", Num::tan, make_unary_numeric);
        define_with!(self, "asin", Num::asin, make_unary_numeric);
        define_with!(self, "acos", Num::acos, make_unary_numeric);
        define_with!(self, "atan", Num::atan, make_unary_numeric);
        define_with!(self, "atan2", Num::atan2, make_binary_numeric);

        // unit conversions
        define_with!(self, "to-degrees", Num::to_degrees, make_unary_numeric);
        define_with!(self, "to-radians", Num::to_radians, make_unary_numeric);

        self
    }
}
