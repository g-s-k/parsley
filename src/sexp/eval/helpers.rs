use std::collections::HashSet;
use std::fmt::Write;
use std::rc::Rc;

use super::super::super::primitives::proc::Procedure::Ctx;
use super::super::super::{Context, Error, Primitive, Result};
use super::SExp::{self, Atom, Null, Pair, Vector};

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
        let mut ret = Atom(Primitive::Undefined);
        for expr in self {
            ret = expr.eval(ctx)?;
        }
        Ok(ret)
    }

    pub(crate) fn eval_case(self, ctx: &mut Context) -> Result {
        match self {
            Pair { head, tail } => {
                let else_ = Self::sym("else");
                let hvl = head.eval(ctx)?;

                for case in *tail {
                    if let Pair {
                        head: objs,
                        tail: body,
                    } = case
                    {
                        if *objs == else_ || objs.iter().any(|e| *e == hvl) {
                            return body.eval_begin(ctx);
                        }
                    }
                }

                hvl.eval_case(ctx)
            }
            Atom(_) | Vector(_) => Ok(Atom(Primitive::Undefined)),
            Null => Err(Error::ArityMin {
                expected: 1,
                given: 0,
            }),
        }
    }

    pub(crate) fn eval_cond(self, ctx: &mut Context) -> Result {
        let else_ = Self::sym("else");

        for case in self {
            match case {
                Pair {
                    head: predicate,
                    tail: consequent,
                } => {
                    // TODO: check if `else` clause is actually last
                    if *predicate == else_ {
                        return consequent.eval_begin(ctx);
                    }

                    match predicate.eval(ctx) {
                        Ok(Atom(Primitive::Boolean(false))) => {
                            continue;
                        }
                        Ok(_) => return consequent.eval_begin(ctx),
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
            Null => Err(Error::ArityMin {
                expected: 2,
                given: 0,
            }),
            Atom(a) => Err(Error::NotAList {
                atom: a.to_string(),
            }),
            Vector(_) => Err(Error::NotAList {
                atom: self.to_string(),
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
                        let ev_defn = the_defn.eval(ctx)?;
                        ctx.define(&sym, ev_defn);
                        Ok(Atom(Primitive::Undefined))
                    }
                    Null => {
                        ctx.define(&sym, Atom(Primitive::Undefined));
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
                    let new_fn = defn
                        .cons(fn_params.cons(Atom(Primitive::Symbol(sym.clone()))))
                        .eval_lambda(ctx, true)?;
                    ctx.define(&sym, new_fn);
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
            } => (match condition.eval(ctx)? {
                Atom(Primitive::Boolean(false)) => if_false,
                _ => if_true,
            })
            .eval(ctx),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    fn sym_to_str(&self) -> Option<&str> {
        if let Atom(Primitive::Symbol(s)) = self {
            Some(s)
        } else {
            None
        }
    }

    pub(crate) fn eval_lambda(self, ctx: &mut Context, is_named: bool) -> Result {
        match self {
            Null => Err(Error::ArityMin {
                expected: 2,
                given: 0,
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
                let (name, params) = if is_named {
                    if let Atom(Primitive::Symbol(s)) = *p_h {
                        (Some(s), *p_t)
                    } else {
                        return Err(Error::Type {
                            expected: "symbol",
                            given: p_h.type_of().to_string(),
                        });
                    }
                } else {
                    (None, p_t.cons(*p_h))
                };
                let fn_body = b_t.cons(*b_h);
                Ok(Self::make_proc(name, params, fn_body, ctx))
            }
            Pair {
                head: box Null,
                tail:
                    box Pair {
                        head: b_h,
                        tail: b_t,
                    },
            } => Ok(Self::make_proc(None, Null, b_t.cons(*b_h), ctx)),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    fn make_proc(name: Option<String>, params: Self, fn_body: Self, ctx: &mut Context) -> Self {
        let expected = params.iter().count();
        let mut params_as_set = params
            .iter()
            .filter_map(Self::sym_to_str)
            .collect::<HashSet<_>>();
        if let Some(ref n) = name {
            params_as_set.insert(n);
        }
        let syms_to_close = fn_body
            .iter()
            .flat_map(|e| {
                e.iter()
                    .flat_map(|e| e.iter().flat_map(|e| e.iter().flat_map(Self::iter)))
            })
            .filter_map(Self::sym_to_str)
            .collect::<HashSet<_>>()
            .difference(&params_as_set)
            .cloned()
            .collect::<Vec<_>>();
        let env = ctx.close(syms_to_close);
        Atom(Primitive::Procedure {
            f: Ctx(Rc::new(move |args: Self, the_ctx: &mut Context| {
                // check arity
                let given = args.iter().count();
                if given != expected {
                    return Err(Error::Arity { expected, given });
                }
                // evaluate arguments
                let evalled_args = args
                    .into_iter()
                    .map(|e| e.eval(the_ctx))
                    .collect::<Result>()?;
                // bind arguments to parameters
                the_ctx.push();
                params
                    .iter()
                    .filter_map(Self::sym_to_str)
                    .zip(evalled_args.into_iter())
                    .for_each(|(p, v)| the_ctx.define(p, v));
                // evaluate each body expression
                let result = fn_body.to_owned().eval_begin(the_ctx);
                the_ctx.pop();
                result
            })),
            name,
            env: if env.is_empty() { None } else { Some(env) },
        })
    }

    pub(crate) fn eval_let(self, ctx: &mut Context) -> Result {
        match self {
            Null => Err(Error::ArityMin {
                expected: 2,
                given: 0,
            }),
            Pair {
                head: defn_list,
                tail: statements,
            } => {
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
        match self {
            Pair {
                head,
                tail: box Null,
            } => Ok(*head),
            _ => Err(Error::Type {
                expected: "list",
                given: self.type_of().to_string(),
            }),
        }
    }

    pub(crate) fn eval_set(self, ctx: &mut Context) -> Result {
        match self {
            Null => Err(Error::Arity {
                expected: 2,
                given: 0,
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
                Err(e) => Err(Error::IO(e)),
            }
        } else {
            Err(Error::Syntax {
                exp: self.to_string(),
            })
        }
    }
}
