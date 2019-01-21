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
    pub fn new<T>(func: T, arity: Arity, env: Option<Env>, name: Option<&str>) -> Self
    where
        Func: From<T>,
    {
        Self {
            name: name.map(String::from),
            arity,
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

impl Into<SExp> for Arity {
    fn into(self) -> SExp {
        use Arity::*;

        match self {
            Exact(n) => (n as f64, n as f64).into(),
            Min(n) => (n as f64, false).into(),
            Range(min, max) => (min as f64, max as f64).into(),
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
