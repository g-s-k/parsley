use std::rc::Rc;

use super::super::super::as_atom::AsAtom;
use super::SExp::{self, *};
use super::{Context, LispError, LispResult, Primitive};

impl SExp {
    pub(super) fn eval_and(self, ctx: &mut Context) -> LispResult {
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

    pub(super) fn eval_begin(self, ctx: &mut Context) -> LispResult {
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

    pub(super) fn eval_cond(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating conditional form.");
        let else_ = SExp::make_symbol("else");

        for case in self {
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
                        return consequent.eval(ctx);
                    }

                    match predicate.eval(ctx) {
                        Ok(Atom(Primitive::Boolean(false))) => {
                            continue;
                        }
                        Ok(_) => return consequent.eval(ctx),
                        err => return err,
                    }
                }
                exp => {
                    return Err(LispError::SyntaxError {
                        exp: exp.to_string(),
                    });
                }
            }
        }

        // falls through if no valid predicates found
        Ok(Atom(Primitive::Void))
    }

    pub(super) fn eval_define(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "define".to_string(),
            }),
            Atom(a) => Err(LispError::NotAList {
                atom: a.to_string(),
            }),
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
                    let ev_defn = defn.eval(ctx)?;
                    ctx.define(&sym, ev_defn);
                    Ok(Atom(Primitive::Undefined))
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
                    Ok(Atom(Primitive::Undefined))
                }
                exp => Err(LispError::SyntaxError {
                    exp: exp.to_string(),
                }),
            },
            exp => Err(LispError::SyntaxError {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_if(self, ctx: &mut Context) -> LispResult {
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

    pub(super) fn eval_lambda(self) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "lambda".to_string(),
            }),
            Atom(a) => Err(LispError::NotAList {
                atom: a.to_string(),
            }),
            Pair {
                head: box params,
                tail: box fn_body,
            } => {
                debug!("Creating procedure.");
                Ok(Atom(Primitive::Procedure(Rc::new(move |args| {
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
                }))))
            }
        }
    }

    pub(super) fn eval_let(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "let".to_string(),
            }),
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
                            err => return err,
                        },
                        exp => {
                            return Err(LispError::SyntaxError {
                                exp: exp.to_string(),
                            });
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

                result
            }
            exp => Err(LispError::SyntaxError {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_or(self, ctx: &mut Context) -> LispResult {
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

    pub(super) fn eval_quote(self) -> Self {
        trace!("Evaluating 'quote' expression: {}", self);
        match self {
            Pair {
                box head,
                tail: box Null,
            } => head,
            _ => self,
        }
    }

    pub(super) fn eval_set(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "set!".to_string(),
            }),
            Pair {
                head: box Atom(Primitive::Symbol(sym)),
                tail: box defn,
            } => ctx.set(&sym, defn),
            exp => Err(LispError::SyntaxError {
                exp: exp.to_string(),
            }),
        }
    }
}
