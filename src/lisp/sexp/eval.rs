use std::rc::Rc;

use super::super::as_atom::AsAtom;
use super::{Context, LispError, LispResult, Primitive};
use super::SExp::{self, *};

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
                        result
                    }
                    None => {
                        // handle everything else
                        debug!("Evaluating normal list: {}", new_pair);
                        let evaluated = new_pair
                            .into_iter()
                            .inspect(|e| trace!("Evaluating list member {}", e))
                            .map(|e| e.eval(ctx))
                            .collect::<Result<Vec<_>, LispError>>()?
                            .into_iter()
                            .rev()
                            .collect::<SExp>();

                        trace!("Applying operation: {}", evaluated);
                        evaluated.apply(ctx)
                    }
                }
            }
        }
    }

    fn eval_special_form(self, ctx: &mut Context) -> Option<LispResult> {
        match self {
            Null => None,
            Atom(_) => None,
            Pair { box head, box tail } => match head {
                Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "lambda" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "lambda".to_string(),
                        })),
                        Atom(a) => Some(Err(LispError::NotAList {
                            atom: a.to_string(),
                        })),
                        Pair {
                            head: box params,
                            tail: box fn_body,
                        } => {
                            debug!("Creating procedure.");
                            Some(Ok(Atom(Primitive::Procedure(Rc::new(move |args| {
                                debug!("Formal parameters: {}", params);
                                let bound_params = params
                                    .to_owned()
                                    .into_iter()
                                    .zip(args.into_iter())
                                    .map(|(p, a)| Null.cons(a).cons(p))
                                    .collect();
                                debug!("Bound parameters: {}", bound_params);
                                Ok(fn_body
                                    .to_owned()
                                    .cons(bound_params)
                                    .cons(SExp::make_symbol("let")))
                            })))))
                        }
                    },
                    "define" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "define".to_string(),
                        })),
                        Atom(a) => Some(Err(LispError::NotAList {
                            atom: a.to_string(),
                        })),
                        Pair {
                            head: box head2,
                            tail:
                                box Pair {
                                    head: box defn,
                                    tail: box Null,
                                },
                        } => match head2 {
                            Atom(Primitive::Symbol(sym)) => {
                                debug!("Defining a quanitity with symbol {}", &sym);
                                ctx.define(&sym, defn.clone());
                                Some(Ok(defn))
                            }
                            Pair {
                                head: box Atom(Primitive::Symbol(sym)),
                                tail: box fn_params,
                            } => {
                                debug!("Defining a function with \"define\" syntax.");
                                ctx.define(
                                    &sym,
                                    Null.cons(defn)
                                        .cons(fn_params)
                                        .cons(SExp::make_symbol("lambda")),
                                );
                                Some(Ok(Atom(Primitive::Undefined)))
                            }
                            exp => Some(Err(LispError::SyntaxError {
                                exp: exp.to_string(),
                            })),
                        },
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "set!" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "set!".to_string(),
                        })),
                        Pair {
                            head: box Atom(Primitive::Symbol(sym)),
                            tail: box defn,
                        } => Some(ctx.set(&sym, defn)),
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "let" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "let".to_string(),
                        })),
                        Pair {
                            head: box defn_list,
                            tail: box statements,
                        } => {
                            debug!("Creating a local binding.");
                            ctx.push();

                            for defn in defn_list {
                                match defn {
                                    Pair {
                                        head: box Atom(Primitive::Symbol(key)),
                                        tail:
                                            box Pair {
                                                head: box val,
                                                tail: box Null,
                                            },
                                    } => match val.eval(ctx) {
                                        Ok(result) => ctx.define(&key, result),
                                        err => return Some(err),
                                    },
                                    exp => {
                                        return Some(Err(LispError::SyntaxError {
                                            exp: exp.to_string(),
                                        }));
                                    }
                                }
                            }

                            let mut result = Err(LispError::NullList);

                            for statement in statements {
                                result = statement.eval(ctx);

                                if result.is_err() {
                                    break;
                                }
                            }

                            ctx.pop();

                            Some(result)
                        }
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "cond" => {
                        debug!("Evaluating conditional form.");
                        let else_ = SExp::make_symbol("else");

                        for case in tail {
                            match case {
                                Pair {
                                    head: box predicate,
                                    tail:
                                        box Pair {
                                            head: box consequent,
                                            tail: box Null,
                                        },
                                } => {
                                    // TODO: check if `else` clause is actually last
                                    if predicate == else_ {
                                        return Some(consequent.eval(ctx));
                                    }

                                    match predicate.eval(ctx) {
                                        Ok(Atom(Primitive::Boolean(false))) => {
                                            continue;
                                        }
                                        Ok(_) => return Some(consequent.eval(ctx)),
                                        err => return Some(err),
                                    }
                                }
                                exp => {
                                    return Some(Err(LispError::SyntaxError {
                                        exp: exp.to_string(),
                                    }));
                                }
                            }
                        }

                        // falls through if no valid predicates found
                        Some(Ok(Atom(Primitive::Void)))
                    }
                    "and" => Some(tail.eval_and(ctx)),
                    "begin" => Some(tail.eval_begin(ctx)),
                    "if" => Some(tail.eval_if(ctx)),
                    "or" => Some(tail.eval_or(ctx)),
                    "quote" => Some(Ok(tail.eval_quote())),
                    _ => None,
                },
                _ => None,
            },
        }
    }

    fn eval_begin(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "begin".to_string(),
            }),
            _ => {
                debug!("Evaluating \"begin\" sequence.");
                match self.into_iter().map(|e| e.eval(ctx)).last() {
                    Some(stuff) => stuff,
                    None => Err(LispError::SyntaxError {
                        exp: "something bad happened, idk".to_string(),
                    }),
                }
            }
        }
    }

    fn eval_and(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating 'and' expression.");
        let mut state = true.as_atom();

        for element in self {
            state = element.eval(ctx)?;

            if let Atom(Primitive::Boolean(false)) = state {
                break;
            }
        }

        Ok(state)
    }

    fn eval_or(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating 'or' expression.");
        for element in self {
            match element.eval(ctx)? {
                Atom(Primitive::Boolean(false)) => (),
                exp => {
                    return Ok(exp);
                }
            }
        }

        Ok(false.as_atom())
    }

    fn eval_if(self, ctx: &mut Context) -> LispResult {
        match self {
            Pair {
                head: box condition,
                tail:
                    box Pair {
                        head: box if_true,
                        tail:
                            box Pair {
                                head: box if_false,
                                tail: box Null,
                            },
                    },
            } => {
                debug!("Evaluating 'if' expression.");
                (match condition.eval(ctx)? {
                    Atom(Primitive::Boolean(false)) => if_false,
                    _ => if_true,
                })
                .eval(ctx)
            }
            exp => Err(LispError::SyntaxError {
                exp: exp.to_string(),
            }),
        }
    }

    fn eval_quote(self) -> Self {
        trace!("Evaluating 'quote' expression: {}", self);
        match self {
            Pair {
                box head,
                tail: box Null,
            } => head,
            _ => self,
        }
    }
}
