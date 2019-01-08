use super::SExp::{self, Atom, Null, Pair};
use super::{Context, Error, Primitive, Result};

impl SExp {
    pub(super) fn eval_and(self, ctx: &mut Context) -> Result {
        debug!("Evaluating 'and' expression.");
        let mut state = Self::from(true);

        for element in self {
            state = element.eval(ctx)?;

            if let Atom(Primitive::Boolean(false)) = state {
                break;
            }
        }

        Ok(state)
    }

    pub(super) fn eval_begin(self, ctx: &mut Context) -> Result {
        if let Null = self {
            Err(Error::NoArgumentsProvided {
                symbol: "begin".to_string(),
            })
        } else {
            debug!("Evaluating \"begin\" sequence.");
            match self.into_iter().map(|e| e.eval(ctx)).last() {
                Some(stuff) => stuff,
                None => Err(Error::Syntax {
                    exp: "something bad happened, idk".to_string(),
                }),
            }
        }
    }

    pub(super) fn eval_cond(self, ctx: &mut Context) -> Result {
        debug!("Evaluating conditional form.");
        let else_ = Self::make_symbol("else");

        for case in self {
            match case {
                Pair {
                    head: predicate,
                    tail:
                        box Pair {
                            head: consequent,
                            tail: box Null,
                        },
                } => {
                    // TODO: check if `else` clause is actually last
                    if *predicate == else_ {
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
                    return Err(Error::Syntax {
                        exp: exp.to_string(),
                    });
                }
            }
        }

        // falls through if no valid predicates found
        Ok(Atom(Primitive::Void))
    }

    pub(super) fn eval_define(self, ctx: &mut Context) -> Result {
        match self {
            Null => Err(Error::NoArgumentsProvided {
                symbol: "define".to_string(),
            }),
            Atom(a) => Err(Error::NotAList {
                atom: a.to_string(),
            }),
            Pair {
                head: head2,
                tail: defn,
            } => match *head2 {
                Atom(Primitive::Symbol(sym)) => match *defn {
                    Pair {
                        head: the_defn,
                        tail: box Null,
                    } => {
                        debug!("Defining a quanitity with symbol {}", &sym);
                        let ev_defn = the_defn.eval(ctx)?;
                        ctx.define(&sym, ev_defn);
                        Ok(Atom(Primitive::Undefined))
                    }
                    exp => Err(Error::Syntax {
                        exp: exp.to_string(),
                    }),
                },
                Pair {
                    head: box Atom(Primitive::Symbol(sym)),
                    tail: fn_params,
                } => {
                    debug!("Defining a function with \"define\" syntax.");
                    ctx.define(
                        &sym,
                        defn.cons(*fn_params).cons(Self::make_symbol("lambda")),
                    );
                    Ok(Atom(Primitive::Undefined))
                }
                exp => Err(Error::Syntax {
                    exp: exp.to_string(),
                }),
            },
        }
    }

    pub(super) fn eval_if(self, ctx: &mut Context) -> Result {
        match self {
            Pair {
                head: condition,
                tail:
                    box Pair {
                        head: if_true,
                        tail:
                            box Pair {
                                head: if_false,
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
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_lambda(self) -> Result {
        match self {
            Null => Err(Error::NoArgumentsProvided {
                symbol: "lambda".to_string(),
            }),
            Atom(a) => Err(Error::NotAList {
                atom: a.to_string(),
            }),
            Pair {
                head:
                    box Pair {
                        head: p_h,
                        tail: p_t,
                    },
                tail:
                    box Pair {
                        head: b_h,
                        tail: b_t,
                    },
            } => {
                debug!("Creating procedure.");
                let params = Pair {
                    head: p_h,
                    tail: p_t,
                };
                let expected = params.iter().count();
                let fn_body = Pair {
                    head: b_h,
                    tail: b_t,
                };
                Ok(Self::from(move |args: Self| {
                    info!("Formal parameters: {}", params);
                    // check arity
                    let given = args.iter().count();
                    if given != expected {
                        return Err(Error::Arity { expected, given });
                    }
                    // bind arguments to parameters
                    let bound_params: Self = params
                        .iter()
                        .zip(args.into_iter())
                        .map(|(p, a)| sexp![p.to_owned(), a])
                        .collect();
                    info!("Bound parameters: {}", bound_params);
                    // construct let binding
                    Ok(fn_body
                        .to_owned()
                        .cons(bound_params)
                        .cons(Self::make_symbol("let")))
                }))
            }
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_let(self, ctx: &mut Context) -> Result {
        match self {
            Null => Err(Error::NoArgumentsProvided {
                symbol: "let".to_string(),
            }),
            Pair {
                head: defn_list,
                tail: statements,
            } => {
                debug!("Creating a local binding.");
                ctx.push();

                for defn in *defn_list {
                    match defn {
                        Pair {
                            head: box Atom(Primitive::Symbol(key)),
                            tail:
                                box Pair {
                                    head: val,
                                    tail: box Null,
                                },
                        } => match *val {
                            Null => ctx.define(&key, Null),
                            _ => match val.eval(ctx) {
                                Ok(result) => ctx.define(&key, result),
                                err => return err,
                            },
                        },
                        exp => {
                            return Err(Error::Syntax {
                                exp: exp.to_string(),
                            });
                        }
                    }
                }

                let mut result = Err(Error::NullList);

                for statement in *statements {
                    result = statement.eval(ctx);

                    if result.is_err() {
                        break;
                    }
                }

                ctx.pop();

                result
            }
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_or(self, ctx: &mut Context) -> Result {
        debug!("Evaluating 'or' expression.");
        for element in self {
            match element.eval(ctx)? {
                Atom(Primitive::Boolean(false)) => (),
                exp => {
                    return Ok(exp);
                }
            }
        }

        Ok(false.into())
    }

    pub(super) fn eval_quote(self) -> Result {
        trace!("Evaluating 'quote' expression: {}", self);
        match self {
            Pair {
                head,
                tail: box Null,
            } => Ok(*head),
            _ => Err(Error::Type),
        }
    }

    pub(super) fn eval_set(self, ctx: &mut Context) -> Result {
        match self {
            Null => Err(Error::NoArgumentsProvided {
                symbol: "set!".to_string(),
            }),
            Pair {
                head: box Atom(Primitive::Symbol(sym)),
                tail:
                    box Pair {
                        head: defn,
                        tail: box Null,
                    },
            } => ctx.set(&sym, *defn),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_map(self, ctx: &mut Context) -> Result {
        match self {
            Pair {
                head,
                tail:
                    box Pair {
                        head: expr,
                        tail: box Null,
                    },
            } => expr
                .eval(ctx)?
                .into_iter()
                .map(|e| Null.cons(e).cons((*head).to_owned()).eval(ctx))
                .collect(),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_fold(self, ctx: &mut Context) -> Result {
        match self {
            Pair {
                head,
                tail:
                    box Pair {
                        head: init,
                        tail:
                            box Pair {
                                head: expr,
                                tail: box Null,
                            },
                    },
            } => expr.eval(ctx)?.into_iter().fold(Ok(*init), |a, e| match a {
                Ok(acc) => Null.cons(e).cons(acc).cons((*head).to_owned()).eval(ctx),
                err => err,
            }),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn eval_filter(self, ctx: &mut Context) -> Result {
        match self {
            Pair {
                head: predicate,
                tail:
                    box Pair {
                        head: list,
                        tail: box Null,
                    },
            } => list
                .eval(ctx)?
                .into_iter()
                .filter_map(|e| match sexp![(*predicate).clone(), e.clone()].eval(ctx) {
                    Ok(Atom(Primitive::Boolean(false))) => None,
                    Ok(_) => Some(Ok(e)),
                    err => Some(err),
                })
                .collect(),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(super) fn do_apply(self, ctx: &mut Context) -> Result {
        match self {
            Pair {
                head: op,
                tail:
                    box Pair {
                        head: args,
                        tail: box Null,
                    },
            } => args.eval(ctx)?.cons(*op).eval(ctx),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }
}
