use std::cmp::PartialEq;
use std::fmt;
use std::rc::Rc;

use super::{Context, Env, Error, Primitive, Result, SExp};

pub mod utils;

/// A primitive value that wraps a procedure.
#[derive(Clone)]
pub struct Proc {
    name: Option<String>,
    arity: Arity,
    pub(crate) func: Func,
}

impl Proc {
    pub fn new<T, U, V>(func: T, arity: U, name: Option<V>) -> Self
    where
        Arity: From<U>,
        Func: From<T>,
        String: From<V>,
    {
        Self {
            name: name.map(String::from),
            arity: arity.into(),
            func: func.into(),
        }
    }

    pub fn get_arity(&self) -> SExp {
        self.arity.into()
    }

    pub fn thunk(&self) -> bool {
        self.arity.thunk()
    }

    pub fn check_arity(&self, n_args: usize) -> std::result::Result<(), Error> {
        self.arity.check(n_args)
    }

    pub(crate) fn defer_eval(&self) -> bool {
        if let Func::Ctx(_) = self.func {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_tail(&self) -> bool {
        if let Func::Tail { .. } = self.func {
            true
        } else {
            false
        }
    }

    pub fn apply(&self, args: SExp, ctx: &mut Context) -> Result {
        self.check_arity(args.len())?;

        match &self.func {
            Func::Ctx(f) => f(ctx, args),
            Func::Pure(f) => f(args),
            Func::Tail { .. } => Ok(self.to_owned().into()),
            Func::Lambda { body, envt, params } => {
                // start new scope and bind args to parameters
                ctx.use_env(envt.clone());
                ctx.push();
                params
                    .iter()
                    .zip(args.into_iter())
                    .for_each(|(p, v)| ctx.define(p, v));

                // evaluate each body expression, returning the last as a thunk
                ctx.eval_defer(body)
            }
        }
    }
}

impl PartialEq for Proc {
    fn eq(&self, other: &Self) -> bool {
        match (&self.func, &other.func) {
            (Func::Ctx(p0), Func::Ctx(p1)) => Rc::ptr_eq(&p0, &p1),
            (Func::Pure(p0), Func::Pure(p1)) => Rc::ptr_eq(&p0, &p1),
            (
                Func::Lambda {
                    body: b0, envt: e0, ..
                },
                Func::Lambda {
                    body: b1, envt: e1, ..
                },
            ) => Rc::ptr_eq(&b0, &b1) && Rc::ptr_eq(&e0, &e1),
            _ => false,
        }
    }
}

impl fmt::Debug for Proc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Proc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.name {
            Some(n) => write!(f, "#<procedure:{}>", n),
            None => write!(f, "#<procedure>"),
        }
    }
}

impl From<Proc> for SExp {
    fn from(p: Proc) -> Self {
        SExp::Atom(Primitive::Procedure(p))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Arity {
    min: usize,
    max: Option<usize>,
}

impl Arity {
    fn thunk(&self) -> bool {
        self.min == 0 && self.max == Some(0)
    }

    fn check(&self, given: usize) -> std::result::Result<(), Error> {
        if given < self.min {
            match self.max {
                Some(n) if n == self.min => Err(Error::Arity {
                    expected: self.min,
                    given,
                }),
                _ => Err(Error::ArityMin {
                    expected: self.min,
                    given,
                }),
            }
        } else {
            match self.max {
                None => Ok(()),
                Some(n) if given <= n => Ok(()),
                Some(expected) if expected == self.min => Err(Error::Arity { expected, given }),
                Some(expected) => Err(Error::ArityMax { expected, given }),
            }
        }
    }
}

impl From<usize> for Arity {
    fn from(min: usize) -> Self {
        Self {
            min,
            max: Some(min),
        }
    }
}

impl From<(usize,)> for Arity {
    fn from((min,): (usize,)) -> Self {
        Self { min, max: None }
    }
}

impl From<(usize, usize)> for Arity {
    fn from((min, max): (usize, usize)) -> Self {
        Self {
            min,
            max: Some(max),
        }
    }
}

impl Into<SExp> for Arity {
    fn into(self) -> SExp {
        if let Some(n) = self.max {
            (self.min, n).into()
        } else {
            (self.min, false).into()
        }
    }
}

#[derive(Clone)]
pub enum Func {
    Ctx(Rc<Fn(&mut Context, SExp) -> Result>),
    Pure(Rc<Fn(SExp) -> Result>),
    Lambda {
        body: Rc<SExp>,
        envt: Rc<Env>,
        params: Vec<String>,
    },
    Tail {
        body: Rc<SExp>,
        envt: Rc<Env>,
    },
}

impl From<Rc<dyn Fn(&mut Context, SExp) -> Result>> for Func {
    fn from(f: Rc<dyn Fn(&mut Context, SExp) -> Result>) -> Self {
        Func::Ctx(f)
    }
}

impl From<Rc<dyn Fn(SExp) -> Result>> for Func {
    fn from(f: Rc<dyn Fn(SExp) -> Result>) -> Self {
        Func::Pure(f)
    }
}
