use super::SExp::{self, *};
use super::{Context, LispError, LispResult, Primitive};

mod helpers;

impl SExp {
    /// Evaluate an S-Expression in a context.
    ///
    /// The context will retain any definitions bound during evaluation
    /// (e.g. `define`, `set!`).
    ///
    /// # Examples
    /// ```
    /// use parsley::{AsAtom, Context};
    /// use parsley::SExp::{self, Null};
    ///
    /// let exp = Null.cons(1.0.as_atom())
    ///     .cons(0.0.as_atom())
    ///     .cons(SExp::make_symbol("eq?"));
    /// let mut ctx = Context::base();
    /// let result = exp.eval(&mut ctx);
    /// assert_eq!(result.unwrap(), false.as_atom());
    /// ```
    /// ```
    /// use parsley::{AsAtom, Context};
    /// use parsley::SExp::{self, Null};
    ///
    /// let exp1 = Null.cons(10.0.as_atom())
    ///     .cons(SExp::make_symbol("x"))
    ///     .cons(SExp::make_symbol("define"));
    /// let exp2 = SExp::make_symbol("x");
    ///
    /// let mut ctx = Context::base();
    /// exp1.eval(&mut ctx);
    /// let result = exp2.eval(&mut ctx);
    /// assert_eq!(result.unwrap(), 10.0.as_atom());
    /// ```
    pub fn eval(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NullList),
            Atom(Primitive::Symbol(sym)) => match ctx.get(&sym) {
                None => Err(LispError::UndefinedSymbol { sym }),
                Some(exp) => exp.eval(ctx),
            },
            Atom(_) => Ok(self),
            Pair { .. } => self.eval_pair(ctx),
        }
    }

    fn eval_pair(self, ctx: &mut Context) -> LispResult {
        match self {
            Pair { head, tail } => {
                if let Atom(Primitive::Symbol(sym)) = *head {
                    match sym.as_ref() {
                        "and" => tail.eval_and(ctx),
                        "begin" => tail.eval_begin(ctx),
                        "cond" => tail.eval_cond(ctx),
                        "define" => tail.eval_define(ctx),
                        "if" => tail.eval_if(ctx),
                        "lambda" => tail.eval_lambda(),
                        "let" => tail.eval_let(ctx),
                        "or" => tail.eval_or(ctx),
                        "quote" => Ok(tail.eval_quote()),
                        "set!" => tail.eval_set(ctx),
                        _ => tail.cons(SExp::make_symbol(&sym)).eval_typical_pair(ctx),
                    }
                } else {
                    tail.cons(*head).eval_typical_pair(ctx)
                }
            }
            _ => self.eval_typical_pair(ctx),
        }
    }

    fn eval_typical_pair(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating normal list: {}", self);
        let evaluated = self
            .into_iter()
            .inspect(|e| trace!("Evaluating list member {}", e))
            .map(|e| e.eval(ctx))
            .collect::<Result<SExp, LispError>>()?;

        trace!("Applying operation: {}", evaluated);
        evaluated.apply(ctx)
    }
}
