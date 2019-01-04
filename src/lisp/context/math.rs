use super::utils::*;
use super::Context;

impl Context {
    /// Math functions that are less commonly used. Intended to be layered on top of the base context.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// let mut ctx = &mut Context::base().math();
    /// let mut asrt = |lhs, rhs| {
    ///     assert_eq!(run_in(lhs, ctx).unwrap(), run_in(rhs, ctx).unwrap())
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
        self.define("is-nan", make_unary_numeric(f64::is_nan));
        self.define("is-infinite", make_unary_numeric(f64::is_infinite));
        self.define("is-finite", make_unary_numeric(f64::is_finite));
        self.define("is-positive", make_unary_numeric(f64::is_sign_positive));
        self.define("is-negative", make_unary_numeric(f64::is_sign_negative));

        // rounding etc.
        self.define("floor", make_unary_numeric(f64::floor));
        self.define("ceil", make_unary_numeric(f64::ceil));
        self.define("round", make_unary_numeric(f64::round));
        self.define("trunc", make_unary_numeric(f64::trunc));
        self.define("fract", make_unary_numeric(f64::fract));
        self.define("sign", make_unary_numeric(f64::signum));

        // exponents, roots, and logs
        self.define("recip", make_unary_numeric(f64::recip));
        self.define("sqrt", make_unary_numeric(f64::sqrt));
        self.define("cube-root", make_unary_numeric(f64::cbrt));
        self.define("exp", make_unary_numeric(f64::exp));
        self.define("log", make_unary_numeric(f64::ln));
        self.define("exp-2", make_unary_numeric(f64::exp2));
        self.define("log-2", make_unary_numeric(f64::log2));
        self.define("log-10", make_unary_numeric(f64::log10));
        self.define("log-n", make_binary_numeric(f64::log));

        // trigonometry
        self.define("hypot", make_binary_numeric(f64::hypot));
        self.define("sin", make_unary_numeric(f64::sin));
        self.define("cos", make_unary_numeric(f64::cos));
        self.define("tan", make_unary_numeric(f64::tan));
        self.define("asin", make_unary_numeric(f64::asin));
        self.define("acos", make_unary_numeric(f64::acos));
        self.define("atan", make_unary_numeric(f64::atan));
        self.define("atan2", make_binary_numeric(f64::atan2));

        // unit conversions
        self.define("to-degrees", make_unary_numeric(f64::to_degrees));
        self.define("to-radians", make_unary_numeric(f64::to_radians));

        self
    }
}
