use std::collections::HashMap;
use std::rc::Rc;

use super::super::proc::{Func, Proc};
use super::super::SExp::{self, Atom, Null, Pair};
use super::super::{Error, Ns, Primitive, Result};
use super::Context;

mod tests;

macro_rules! tup_ctx_env {
    ( $name:expr, $proc:expr, $arity:expr ) => {
        (
            $name.to_string(),
            $crate::SExp::from($crate::Proc::new(
                $crate::Func::Ctx(::std::rc::Rc::new($proc)),
                $arity,
                Some($name),
            )),
        )
    };
}

impl Context {
    pub(super) fn core() -> Ns {
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
            tup_ctx_env!("do", Self::eval_do, (3,)),
            tup_ctx_env!("define", Self::eval_define, (1,)),
            tup_ctx_env!("if", Self::eval_if, 3),
            tup_ctx_env!("lambda", |e, c| Self::eval_lambda(e, c, false), (2,)),
            tup_ctx_env!("let", Self::eval_let, (2,)),
            tup_ctx_env!("named-lambda", |e, c| Self::eval_lambda(e, c, true), (2,)),
            tup_ctx_env!("or", Self::eval_or, (0,)),
            tup_ctx_env!("quasiquote", Self::eval_quasiquote, 1),
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
                            return self.eval_defer(&*body);
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
                        return self.eval_defer(&*consequent);
                    }

                    match self.eval(*predicate)? {
                        Atom(Primitive::Boolean(false)) => {
                            continue;
                        }
                        _ => return self.eval_defer(&*consequent),
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

    fn eval_do(&mut self, expr: SExp) -> Result {
        let (vars, rest) = expr.split_car()?;
        let (term, body) = rest.split_car()?;

        // get definitions for loop vars
        let mut var_inits = HashMap::new();
        let mut var_updates = HashMap::new();

        for var in vars {
            match var.split_car()? {
                (Atom(Primitive::Symbol(s)), rest) => match rest.len() {
                    1 => {
                        var_inits.insert(s, rest.car()?);
                    }
                    2 => {
                        let (defn, tail) = rest.split_car()?;
                        var_inits.insert(s.clone(), defn);
                        var_updates.insert(s, tail.car()?);
                    }
                    0 => {
                        return Err(Error::ArityMin {
                            expected: 1,
                            given: 0,
                        });
                    }
                    given => return Err(Error::ArityMax { expected: 2, given }),
                },
                (other, _) => {
                    return Err(Error::Type {
                        expected: "symbol",
                        given: other.type_of().to_string(),
                    });
                }
            }
        }

        // termination condition and return value
        let (cond, return_expr) = term.split_car()?;

        // add definitions to environment
        self.push();
        self.cont.borrow().env().extend(var_inits);

        let result = 'eval: loop {
            // do each step
            for exp in body.iter() {
                if let Err(err) = self.eval(exp.to_owned()) {
                    break 'eval Err(err);
                }
            }

            // check termination condition, update vars if necessary
            match self.eval(cond.clone()) {
                Ok(Atom(Primitive::Boolean(false))) => {
                    // we don't want the new values to be in place while we
                    // evaluate subsequent step variables, so we hold them in a
                    // temporary map, then insert them all at once
                    let mut new_map = HashMap::new();
                    for (key, upd) in &var_updates {
                        let new_val = match self.eval(upd.to_owned()) {
                            Ok(v) => v,
                            err => break 'eval err,
                        };
                        new_map.insert(key.to_string(), new_val);
                    }
                    self.cont.borrow().env().extend(new_map);
                }
                Ok(_) => break 'eval self.eval_begin(return_expr),
                err => break 'eval err,
            }
        };

        self.pop();
        result
    }

    fn eval_if(&mut self, expr: SExp) -> Result {
        let (condition, cdr) = expr.split_car()?;
        let (if_true, cdr) = cdr.split_car()?;
        let (if_false, _) = cdr.split_car()?;

        let cevl = self.eval(condition)?;
        Ok(self.defer(if let Atom(Primitive::Boolean(false)) = cevl {
            if_false
        } else {
            if_true
        }))
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

        let str_sig = signature
            .into_iter()
            .map(|e| {
                if let Atom(Primitive::Symbol(sym)) = e {
                    Ok(sym)
                } else {
                    Err(Error::Type {
                        expected: "symbol",
                        given: e.type_of().to_string(),
                    })
                }
            })
            .collect::<std::result::Result<Vec<_>, Error>>()?;

        if is_named {
            Ok(self.make_proc(Some(&str_sig[0]), str_sig[1..].to_vec(), fn_body))
        } else {
            Ok(self.make_proc(None, str_sig, fn_body))
        }
    }

    fn make_proc(&self, name: Option<&str>, params: Vec<String>, fn_body: SExp) -> SExp {
        let expected = params.len();
        SExp::from(Proc::new(
            Func::Lambda {
                body: Rc::new(fn_body),
                envt: self.cont.borrow().env(),
                params,
            },
            expected,
            name,
        ))
    }

    pub(super) fn defer(&self, expr: SExp) -> SExp {
        SExp::from(Proc::new::<_, _, &str>(
            Func::Tail {
                body: Rc::new(expr),
                envt: self.cont.borrow().env(),
            },
            0,
            None,
        ))
    }

    fn eval_let(&mut self, expr: SExp) -> Result {
        let (defn_list, statements) = expr.split_car()?;

        if let Atom(Primitive::Symbol(let_name)) = defn_list {
            let (defn_list, statements) = statements.split_car()?;

            let (params, inits): (Vec<_>, Vec<_>) = defn_list
                .into_iter()
                .map(|e| {
                    let (s, r) = e.split_car()?;
                    let d = r.car()?;
                    let sym = if let Atom(Primitive::Symbol(sym)) = s {
                        sym
                    } else {
                        return Err(Error::Type {
                            expected: "symbol",
                            given: s.type_of().to_string(),
                        });
                    };
                    Ok((sym, d))
                })
                .collect::<std::result::Result<Vec<(String, SExp)>, Error>>()?
                .into_iter()
                .unzip();

            self.push();
            let proc = self.make_proc(Some(&let_name), params, statements);
            self.define(&let_name, proc);
            let applic = SExp::from(inits).cons(Atom(Primitive::Symbol(let_name)));
            let result = self.eval(applic);
            self.pop();
            result
        } else {
            self.push();

            for defn in defn_list {
                let err = self.eval_define(defn);

                if err.is_err() {
                    self.pop();
                    return err;
                }
            }

            let result = self.eval_defer(&statements);

            self.pop();
            result
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

    fn eval_quasiquote(&mut self, expr: SExp) -> Result {
        match expr.car()? {
            p @ Pair { .. } => p
                .into_iter()
                .map(|sub_expr| match sub_expr {
                    Pair { head, tail } => match *head {
                        Atom(Primitive::Symbol(ref s)) if s == "unquote" => self.eval(tail.car()?),
                        _ => Ok(tail.cons(*head)),
                    },
                    _ => Ok(sub_expr),
                })
                .collect::<Result>(),
            other => Ok(other),
        }
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
