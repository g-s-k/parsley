use std::fmt;
use std::rc::Rc;

use super::{Context, Env, Primitive, Result, SExp};

pub type CtxFunc = Rc<Fn(&mut Context, SExp) -> Result>;
pub type PureFunc = Rc<Fn(SExp) -> Result>;

#[derive(Clone)]
pub struct Proc {
    name: Option<String>,
    arity: Arity,
    env: Option<Env>,
    func: Func,
}

impl Proc {
    fn new<T>(func: T, arity: Arity, env: Option<Env>, name: Option<&str>) -> Self
    where
        T: Into<Func>,
    {
        Self {
            name: name.map(String::from),
            arity,
            env,
            func: func.into(),
        }
    }
}

impl fmt::Display for Proc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.name {
            Some(n) => write!(f, "#<procedure:{}>", n),
            None => write!(f, "#<procedure>"),
        }
    }
}

impl Into<SExp> for Proc {
    fn into(self) -> SExp {
        SExp::Atom(Primitive::Procedure(self))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Arity {
    Exact(usize),
    Minimum(usize),
    Range(usize, usize),
}

impl Into<SExp> for Arity {
    fn into(self) -> SExp {
        use Arity::*;

        match self {
            Exact(n) => (n as f64, n as f64).into(),
            Minimum(n) => (n as f64, false).into(),
            Range(min, max) => (min as f64, max as f64).into(),
        }
    }
}

#[derive(Clone)]
pub enum Func {
    Ctx(Rc<CtxFunc>),
    Pure(Rc<PureFunc>),
}

impl From<CtxFunc> for Func {
    fn from(f: CtxFunc) -> Self {
        Func::Ctx(Rc::new(f))
    }
}

impl From<PureFunc> for Func {
    fn from(f: PureFunc) -> Self {
        Func::Pure(Rc::new(f))
    }
}
