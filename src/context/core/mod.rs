use std::collections::HashSet;
use std::rc::Rc;

use super::super::proc::{Func, Proc};
use super::super::SExp::{self, Atom, Null, Pair};
use super::super::{Env, Error, Primitive, Result};
use super::Context;

mod tests;

macro_rules! tup_ctx_env {
    ( $name:expr, $proc:expr, $arity:expr ) => {
        (
            $name.to_string(),
            $crate::SExp::from($crate::Proc::new(
                $crate::Func::Ctx(::std::rc::Rc::new($proc)),
                $arity,
                None,
                Some($name),
            )),
        )
    };
}

impl Context {
    pub(super) fn core() -> Env {
        [
            tup_ctx_env!(
                "eval",
                |c: &mut Self, e: SExp| {
                    let first_layer = c.eval(e.car()?)?;
                    c.eval(first_layer)
                },
                1
            ),
            tup_ctx_env!("apply", Self::do_apply, 2),
            tup_ctx_env!("and", Self::eval_and, (0,)),
            tup_ctx_env!("begin", Self::eval_begin, (0,)),
            tup_ctx_env!("case", Self::eval_case, (2,)),
            tup_ctx_env!("cond", Self::eval_cond, (0,)),
            tup_ctx_env!("define", Self::eval_define, (1,)),
            tup_ctx_env!("if", Self::eval_if, 3),
            tup_ctx_env!("lambda", |e, c| Self::eval_lambda(e, c, false), (2,)),
            tup_ctx_env!("let", Self::eval_let, (2,)),
            tup_ctx_env!("named-lambda", |e, c| Self::eval_lambda(e, c, true), (2,)),
            tup_ctx_env!("or", Self::eval_or, (0,)),
            tup_ctx_env!("quote", Self::eval_quote, 1),
            tup_ctx_env!("set!", Self::eval_set, 2),
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
            Atom(_) => Ok(Atom(Primitive::Undefined)),
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
        let (signature, defn) = expr.split_car()?;

        let (sym, the_defn) = match signature {
            // procedure
            Pair { head, tail } => {
                let sym = match *head {
                    Atom(Primitive::Symbol(ref sym)) => sym.to_owned(),
                    other => {
                        return Err(Error::Type {
                            expected: "symbol",
                            given: other.type_of().to_string(),
                        });
                    }
                };

                (sym, self.eval_lambda(defn.cons(tail.cons(*head)), true)?)
            }
            // simple value - can be nothing or something
            Atom(Primitive::Symbol(sym)) => {
                match defn.len() {
                    0 | 1 => (),
                    given => return Err(Error::ArityMax { expected: 1, given }),
                }

                match defn {
                    Null => (sym, Atom(Primitive::Undefined)),
                    p @ Pair { .. } => (sym, self.eval(p.car()?)?),
                    other => (sym, self.eval(other)?),
                }
            }
            other => {
                return Err(Error::Type {
                    expected: "symbol",
                    given: other.type_of().to_string(),
                });
            }
        };

        // actually persist the definition to the environment
        self.define(&sym, the_defn);
        Ok(Atom(Primitive::Undefined))
    }

    fn eval_if(&mut self, expr: SExp) -> Result {
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

    fn eval_lambda(&mut self, expr: SExp, is_named: bool) -> Result {
        let (signature, fn_body) = expr.split_car()?;

        match signature {
            Pair { .. } => (),
            other => {
                return Err(Error::Type {
                    expected: "list",
                    given: other.type_of().to_string(),
                });
            }
        }

        for sym in signature.iter() {
            if let Atom(Primitive::Symbol(_)) = sym {
            } else {
                return Err(Error::Type {
                    expected: "symbol",
                    given: sym.type_of().to_string(),
                });
            }
        }

        if is_named {
            let (name, signature) = signature.split_car()?;
            let name = name.sym_to_str().unwrap();
            Ok(self.make_proc(Some(name), signature, fn_body))
        } else {
            Ok(self.make_proc(None, signature, fn_body))
        }
    }

    fn make_proc(&mut self, name: Option<&str>, params: SExp, fn_body: SExp) -> SExp {
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
        SExp::from(Proc::new(
            Func::Ctx(Rc::new(move |the_ctx: &mut Self, args: SExp| {
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
            expected,
            if env.is_empty() { None } else { Some(env) },
            name,
        ))
    }

    fn eval_let(&mut self, expr: SExp) -> Result {
        let (defn_list, statements) = expr.split_car()?;

        self.push();

        for defn in defn_list {
            let err = self.eval_define(defn);

            if err.is_err() {
                self.pop();
                return err;
            }
        }

        let result = self.eval_begin(statements);

        self.pop();
        result
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
            Pair { .. } => Ok(expr.car()?),
            Null => Err(Error::Type {
                expected: "list",
                given: expr.type_of().to_string(),
            }),
            _ => Ok(expr),
        }
    }

    fn eval_set(&mut self, expr: SExp) -> Result {
        let (name, tail) = expr.split_car()?;

        let sym = match name {
            Atom(Primitive::Symbol(sym)) => sym,
            other => {
                return Err(Error::Type {
                    expected: "symbol",
                    given: other.type_of().to_string(),
                });
            }
        };

        self.set(&sym, tail.car()?)
    }

    fn do_apply(&mut self, expr: SExp) -> Result {
        let (op, tail) = expr.split_car()?;

        let args = self.eval(tail.car()?)?;
        self.eval(args.cons(op))
    }
}
