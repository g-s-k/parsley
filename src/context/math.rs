use super::utils::*;
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
        define_with!(self, "is-nan", f64::is_nan, make_unary_numeric);
        define_with!(self, "is-infinite", f64::is_infinite, make_unary_numeric);
        define_with!(self, "is-finite", f64::is_finite, make_unary_numeric);
        define_with!(
            self,
            "is-positive",
            f64::is_sign_positive,
            make_unary_numeric
        );
        define_with!(
            self,
            "is-negative",
            f64::is_sign_negative,
            make_unary_numeric
        );

        // rounding etc.
        define_with!(self, "floor", f64::floor, make_unary_numeric);
        define_with!(self, "ceil", f64::ceil, make_unary_numeric);
        define_with!(self, "round", f64::round, make_unary_numeric);
        define_with!(self, "trunc", f64::trunc, make_unary_numeric);
        define_with!(self, "fract", f64::fract, make_unary_numeric);
        define_with!(self, "sign", f64::signum, make_unary_numeric);

        // exponents, roots, and logs
        define_with!(self, "recip", f64::recip, make_unary_numeric);
        define_with!(self, "sqrt", f64::sqrt, make_unary_numeric);
        define_with!(self, "cube-root", f64::cbrt, make_unary_numeric);
        define_with!(self, "exp", f64::exp, make_unary_numeric);
        define_with!(self, "log", f64::ln, make_unary_numeric);
        define_with!(self, "exp-2", f64::exp2, make_unary_numeric);
        define_with!(self, "log-2", f64::log2, make_unary_numeric);
        define_with!(self, "log-10", f64::log10, make_unary_numeric);
        define_with!(self, "log-n", f64::log, make_binary_numeric);

        // trigonometry
        define_with!(self, "hypot", f64::hypot, make_binary_numeric);
        define_with!(self, "sin", f64::sin, make_unary_numeric);
        define_with!(self, "cos", f64::cos, make_unary_numeric);
        define_with!(self, "tan", f64::tan, make_unary_numeric);
        define_with!(self, "asin", f64::asin, make_unary_numeric);
        define_with!(self, "acos", f64::acos, make_unary_numeric);
        define_with!(self, "atan", f64::atan, make_unary_numeric);
        define_with!(self, "atan2", f64::atan2, make_binary_numeric);

        // unit conversions
        define_with!(self, "to-degrees", f64::to_degrees, make_unary_numeric);
        define_with!(self, "to-radians", f64::to_radians, make_unary_numeric);

        self
    }
}
