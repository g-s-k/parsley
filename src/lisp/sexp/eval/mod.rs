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
            Pair { box head, box tail } => {
                // handle special functions
                let new_pair = tail.to_owned().cons(head.to_owned());
                match new_pair.clone().eval_special_form(ctx) {
                    Some(result) => {
                        debug!("Special form finished evaluating.");
                        trace!("Result of special form eval: {:?}", result);
                        result
                    }
                    None => {
                        // handle everything else
                        debug!("Evaluating normal list: {}", new_pair);
                        let evaluated = new_pair
                            .into_iter()
                            .inspect(|e| trace!("Evaluating list member {}", e))
                            .map(|e| e.eval(ctx))
                            .collect::<Result<SExp, LispError>>()?;

                        trace!("Applying operation: {}", evaluated);
                        evaluated.apply(ctx)
                    }
                }
            }
        }
    }

    fn eval_special_form(self, ctx: &mut Context) -> Option<LispResult> {
        match self {
            Pair {
                head: box Atom(Primitive::Symbol(sym)),
                box tail,
            } => match sym.as_ref() {
                "and" => Some(tail.eval_and(ctx)),
                "begin" => Some(tail.eval_begin(ctx)),
                "cond" => Some(tail.eval_cond(ctx)),
                "define" => Some(tail.eval_define(ctx)),
                "if" => Some(tail.eval_if(ctx)),
                "lambda" => Some(tail.eval_lambda()),
                "let" => Some(tail.eval_let(ctx)),
                "or" => Some(tail.eval_or(ctx)),
                "quote" => Some(Ok(tail.eval_quote())),
                "set!" => Some(tail.eval_set(ctx)),
                _ => None,
            },
            _ => None,
        }
    }
}
