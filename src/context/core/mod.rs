use std::collections::HashSet;
use std::rc::Rc;

use super::super::SExp::{self, Atom, Null, Pair, Vector};
use super::super::{Env, Error, Primitive, Result};
use super::Context;

mod tests;

macro_rules! tup_ctx_env {
    ( $name:expr, $proc:expr ) => {
        (
            $name.to_string(),
            $crate::SExp::ctx_proc($proc, Some($name)),
        )
    };
}

impl Context {
    pub(super) fn core() -> Env {
        [
            tup_ctx_env!("eval", |c, e| {
                let first_layer = c.eval(e.car()?)?;
                c.eval(first_layer)
            }),
            tup_ctx_env!("apply", Self::do_apply),
            tup_ctx_env!("and", Self::eval_and),
            tup_ctx_env!("begin", Self::eval_begin),
            tup_ctx_env!("case", Self::eval_case),
            tup_ctx_env!("cond", Self::eval_cond),
            tup_ctx_env!("define", Self::eval_define),
            tup_ctx_env!("if", Self::eval_if),
            tup_ctx_env!("lambda", |e, c| Self::eval_lambda(e, c, false)),
            tup_ctx_env!("let", Self::eval_let),
            tup_ctx_env!("named-lambda", |e, c| Self::eval_lambda(e, c, true)),
            tup_ctx_env!("or", Self::eval_or),
            tup_ctx_env!("quote", Self::eval_quote),
            tup_ctx_env!("set!", Self::eval_set),
        ]
        .iter()
        .cloned()
        .collect()
    }

    fn eval_and(&mut self, expr: SExp) -> Result {
        let mut state = SExp::from(true);

        for element in expr {
            state = self.eval(element)?;

            if let Atom(Primitive::Boolean(false)) = state {
                break;
            }
        }

        Ok(state)
    }

    fn eval_begin(&mut self, expr: SExp) -> Result {
        let mut ret = Atom(Primitive::Undefined);
        for exp in expr {
            ret = self.eval(exp)?;
        }
        Ok(ret)
    }

    fn eval_case(&mut self, expr: SExp) -> Result {
        match expr {
            Pair { head, tail } => {
                let else_ = SExp::sym("else");
                let hvl = self.eval(*head)?;

                for case in *tail {
                    if let Pair {
                        head: objs,
                        tail: body,
                    } = case
                    {
                        if *objs == else_ || objs.iter().any(|e| *e == hvl) {
                            return self.eval_begin(*body);
                        }
                    }
                }

                self.eval_case(hvl)
            }
            Atom(_) | Vector(_) => Ok(Atom(Primitive::Undefined)),
            Null => Err(Error::ArityMin {
                expected: 1,
                given: 0,
            }),
        }
    }

    fn eval_cond(&mut self, expr: SExp) -> Result {
        let else_ = SExp::sym("else");

        for case in expr {
            match case {
                Pair {
                    head: predicate,
                    tail: consequent,
                } => {
                    // TODO: check if `else` clause is actually last
                    if *predicate == else_ {
                        return self.eval_begin(*consequent);
                    }

                    match self.eval(*predicate)? {
                        Atom(Primitive::Boolean(false)) => {
                            continue;
                        }
                        _ => return self.eval_begin(*consequent),
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

    fn eval_define(&mut self, expr: SExp) -> Result {
        match expr {
            Null => Err(Error::ArityMin {
                expected: 2,
                given: 0,
            }),
            Atom(a) => Err(Error::NotAList {
                atom: a.to_string(),
            }),
            Vector(_) => Err(Error::NotAList {
                atom: expr.to_string(),
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
                        let df = self.eval(*the_defn)?;
                        self.define(&sym, df);
                        Ok(Atom(Primitive::Undefined))
                    }
                    Null => {
                        self.define(&sym, Atom(Primitive::Undefined));
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
                    let func = self.eval_lambda(
                        defn.cons(fn_params.cons(Atom(Primitive::Symbol(sym.clone())))),
                        true,
                    )?;
                    self.define(&sym, func);
                    Ok(Atom(Primitive::Undefined))
                }
                exp => Err(Error::Syntax {
                    exp: exp.to_string(),
                }),
            },
        }
    }

    fn eval_if(&mut self, expr: SExp) -> Result {
        match expr.len() {
            3 => {
                let (condition, cdr) = expr.split_car()?;
                let (if_true, cdr) = cdr.split_car()?;
                let (if_false, _) = cdr.split_car()?;

                let cevl = self.eval(condition)?;
                self.eval(if let Atom(Primitive::Boolean(false)) = cevl {
                    if_false
                } else {
                    if_true
                })
            }
            given => Err(Error::Arity { expected: 3, given }),
        }
    }

    fn eval_lambda(&mut self, expr: SExp, is_named: bool) -> Result {
        match expr {
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
                Ok(self.make_proc(name, params, fn_body))
            }
            Pair {
                head: box Null,
                tail:
                    box Pair {
                        head: b_h,
                        tail: b_t,
                    },
            } => Ok(self.make_proc(None, Null, b_t.cons(*b_h))),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    fn make_proc(&mut self, name: Option<String>, params: SExp, fn_body: SExp) -> SExp {
        use super::super::primitives::proc::Procedure::Ctx;

        let expected = params.iter().count();
        let mut params_as_set = params
            .iter()
            .filter_map(SExp::sym_to_str)
            .collect::<HashSet<_>>();
        if let Some(ref n) = name {
            params_as_set.insert(n);
        }
        let syms_to_close = fn_body
            .iter()
            .flat_map(|e| {
                e.iter()
                    .flat_map(|e| e.iter().flat_map(|e| e.iter().flat_map(SExp::iter)))
            })
            .filter_map(SExp::sym_to_str)
            .collect::<HashSet<_>>()
            .difference(&params_as_set)
            .cloned()
            .collect::<Vec<_>>();
        let env = self.close(syms_to_close);
        Atom(Primitive::Procedure {
            f: Ctx(Rc::new(move |the_ctx: &mut Self, args: SExp| {
                // check arity
                let given = args.iter().count();
                if given != expected {
                    return Err(Error::Arity { expected, given });
                }
                // evaluate arguments
                let evalled_args = args
                    .into_iter()
                    .map(|e| the_ctx.eval(e))
                    .collect::<Result>()?;
                // bind arguments to parameters
                the_ctx.push();
                params
                    .iter()
                    .filter_map(SExp::sym_to_str)
                    .zip(evalled_args.into_iter())
                    .for_each(|(p, v)| the_ctx.define(p, v));
                // evaluate each body expression
                let result = the_ctx.eval_begin(fn_body.to_owned());
                the_ctx.pop();
                result
            })),
            name,
            env: if env.is_empty() { None } else { Some(env) },
        })
    }

    fn eval_let(&mut self, expr: SExp) -> Result {
        match expr {
            Null => Err(Error::ArityMin {
                expected: 2,
                given: 0,
            }),
            Pair {
                head: defn_list,
                tail: statements,
            } => {
                self.push();

                for defn in *defn_list {
                    match defn {
                        Pair {
                            head: box Atom(Primitive::Symbol(key)),
                            tail:
                                box Pair {
                                    head: val,
                                    tail: box Null,
                                },
                        } => {
                            if let Null = *val {
                                self.define(&key, Null)
                            } else {
                                let inter = self.eval(*val)?;
                                self.define(&key, inter)
                            }
                        }
                        exp => {
                            return Err(Error::Syntax {
                                exp: exp.to_string(),
                            });
                        }
                    }
                }

                let mut result = Err(Error::NullList);

                for statement in *statements {
                    result = self.eval(statement);

                    if result.is_err() {
                        break;
                    }
                }

                self.pop();

                result
            }
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    fn eval_or(&mut self, expr: SExp) -> Result {
        for element in expr {
            match self.eval(element)? {
                Atom(Primitive::Boolean(false)) => continue,
                exp => {
                    return Ok(exp);
                }
            }
        }

        Ok(false.into())
    }

    fn eval_quote(&mut self, expr: SExp) -> Result {
        match expr {
            Pair {
                head,
                tail: box Null,
            } => Ok(*head),
            _ => Err(Error::Type {
                expected: "list",
                given: expr.type_of().to_string(),
            }),
        }
    }

    fn eval_set(&mut self, expr: SExp) -> Result {
        match expr {
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
            } => self.set(&sym, *defn),
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }

    fn do_apply(&mut self, expr: SExp) -> Result {
        match expr {
            Pair {
                head: op,
                tail:
                    box Pair {
                        head: args,
                        tail: box Null,
                    },
            } => {
                let inter = self.eval(*args)?.cons(*op);
                self.eval(inter)
            }
            exp => Err(Error::Syntax {
                exp: exp.to_string(),
            }),
        }
    }
}
