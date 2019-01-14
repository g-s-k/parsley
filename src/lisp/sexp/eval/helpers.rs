use std::fmt::Write;
use std::rc::Rc;

use super::SExp::{self, Atom, Null, Pair};
use super::{Context, Error, Primitive, Result};

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
        .replace("\\r", "\r")
        .replace("\\0", "\0")
        .replace("\\\"", "\"")
}

impl SExp {
    pub(crate) fn eval_and(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_begin(self, ctx: &mut Context) -> Result {
        debug!("Evaluating \"begin\" sequence.");
        let mut ret = Atom(Primitive::Undefined);
        for expr in self {
            ret = expr.eval(ctx)?;
        }
        Ok(ret)
    }

    pub(crate) fn eval_cond(self, ctx: &mut Context) -> Result {
        debug!("Evaluating conditional form.");
        let else_ = Self::sym("else");

        for case in self {
            match case {
                Pair {
                    head: predicate,
                    tail: consequent,
                } => {
                    let wrapped_consequent = consequent.cons(Self::sym("begin"));
                    // TODO: check if `else` clause is actually last
                    if *predicate == else_ {
                        return wrapped_consequent.eval(ctx);
                    }

                    match predicate.eval(ctx) {
                        Ok(Atom(Primitive::Boolean(false))) => {
                            continue;
                        }
                        Ok(_) => return wrapped_consequent.eval(ctx),
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

    pub(crate) fn eval_define(self, ctx: &mut Context) -> Result {
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
                    ctx.define(&sym, defn.cons(*fn_params).cons(Self::sym("lambda")));
                    Ok(Atom(Primitive::Undefined))
                }
                exp => Err(Error::Syntax {
                    exp: exp.to_string(),
                }),
            },
        }
    }

    pub(crate) fn eval_if(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_lambda(self, _: &mut Context, is_named: bool) -> Result {
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
                let (name, params) = if is_named {
                    if let Atom(Primitive::Symbol(s)) = *p_h {
                        (Some(s), *p_t)
                    } else {
                        return Err(Error::Type);
                    }
                } else {
                    (None, p_t.cons(*p_h))
                };
                let expected = params.iter().count();
                let fn_body = b_t.cons(*b_h);
                Ok(Atom(Primitive::Procedure {
                    f: Rc::new(move |args: Self| {
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
                            .map(|(p, a)| Null.cons(a).cons(p.to_owned()))
                            .collect();
                        info!("Bound parameters: {}", bound_params);
                        // construct let binding
                        Ok(fn_body.to_owned().cons(bound_params).cons(Self::sym("let")))
                    }),
                    name,
                }))
            }
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(crate) fn eval_let(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_or(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_quote(self, _: &mut Context) -> Result {
        trace!("Evaluating 'quote' expression: {}", self);
        match self {
            Pair {
                head,
                tail: box Null,
            } => Ok(*head),
            _ => Err(Error::Type),
        }
    }

    pub(crate) fn eval_set(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_map(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_fold(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn eval_filter(self, ctx: &mut Context) -> Result {
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
                .filter_map(
                    |e| match Null.cons(e.clone()).cons((*predicate).clone()).eval(ctx) {
                        Ok(Atom(Primitive::Boolean(false))) => None,
                        Ok(_) => Some(Ok(e)),
                        err => Some(err),
                    },
                )
                .collect(),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    pub(crate) fn do_apply(self, ctx: &mut Context) -> Result {
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

    pub(crate) fn do_print(self, ctx: &mut Context, newline: bool, debug: bool) -> Result {
        if let Pair {
            head,
            tail: box Null,
        } = self
        {
            let ending = if newline { "\n" } else { "" };
            let hevl = head.eval(ctx)?;
            let unescaped = unescape(&if debug {
                format!("{:?}{}", hevl, ending)
            } else {
                format!("{}{}", hevl, ending)
            });
            match write!(ctx, "{}", unescaped) {
                Ok(()) => Ok(Atom(Primitive::Undefined)),
                Err(_) => Err(Error::Type),
            }
        } else {
            Err(Error::Syntax {
                exp: self.to_string(),
            })
        }
    }
}
