#![allow(clippy::cast_precision_loss)]

use std::fmt;
use std::rc::Rc;

use super::{Context, Env, Primitive, Result, SExp};

#[derive(Clone)]
pub struct Proc {
    name: Option<String>,
    arity: Arity,
    pub(crate) env: Option<Env>,
    pub(crate) func: Func,
}

impl Proc {
    pub fn new<T, U>(func: T, arity: U, env: Option<Env>, name: Option<&str>) -> Self
    where
        Arity: From<U>,
        Func: From<T>,
    {
        Self {
            name: name.map(String::from),
            arity: arity.into(),
            env,
            func: func.into(),
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
pub enum Arity {
    Exact(usize),
    Min(usize),
    Range(usize, usize),
}

impl From<usize> for Arity {
    fn from(n: usize) -> Self {
        Arity::Exact(n)
    }
}

impl From<(usize,)> for Arity {
    fn from((n,): (usize,)) -> Self {
        Arity::Min(n)
    }
}

impl From<(usize, usize)> for Arity {
    fn from((n0, n1): (usize, usize)) -> Self {
        if n0 == n1 {
            Arity::Exact(n0)
        } else {
            Arity::Range(n0, n1)
        }
    }
}

impl Into<SExp> for Arity {
    fn into(self) -> SExp {
        match self {
            Arity::Exact(n) => (n as f64, n as f64).into(),
            Arity::Min(n) => (n as f64, false).into(),
            Arity::Range(min, max) => (min as f64, max as f64).into(),
        }
    }
}

#[derive(Clone)]
pub enum Func {
    Ctx(Rc<Fn(&mut Context, SExp) -> Result>),
    Pure(Rc<Fn(SExp) -> Result>),
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
