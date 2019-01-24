use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::rc::Rc;

use super::{Error, Result, SExp};

/// A type to represent an execution environment.
pub type Ns = HashMap<String, SExp>;

type Link = Option<Rc<Env>>;

#[derive(Debug, Default)]
pub struct Env {
    env: RefCell<Ns>,
    parent: Link,
}

impl Env {
    pub fn new(parent: Link) -> Self {
        Self {
            parent,
            ..Self::default()
        }
    }

    pub fn parent(&self) -> Link {
        self.parent.clone()
    }

    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn iter(&self) -> Iter {
        Iter(Some(self))
    }

    pub fn len(&self) -> usize {
        self.parent().into_iter().count() + 1
    }

    pub fn extend(&self, other: Ns) {
        self.env.borrow_mut().extend(other.into_iter());
    }

    pub fn get(&self, key: &str) -> Option<SExp> {
        for ns in self.iter() {
            if let Some(val) = ns.env.borrow().get(key) {
                return Some(val.clone());
            }
        }

        None
    }

    pub fn define(&self, key: &str, val: SExp) {
        self.env.borrow_mut().insert(key.to_string(), val);
    }

    pub fn set(&self, key: &str, val: SExp) -> Result {
        let possible_err = Error::UndefinedSymbol {
            sym: key.to_string(),
        };

        for ns in self.iter() {
            if ns.env.borrow().get(key).is_some() {
                return ns
                    .env
                    .borrow_mut()
                    .insert(key.to_string(), val)
                    .ok_or(possible_err);
            }
        }

        Err(possible_err)
    }
}

pub struct Iter<'a>(Option<&'a Env>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Env;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.take();

        if let Some(rc) = ret {
            if let Some(p) = &rc.parent {
                self.0 = Some(&p);
            }
        }

        ret
    }
}

impl IntoIterator for Env {
    type Item = Rc<Self>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Some(self.into_rc()))
    }
}

pub struct IntoIter(Link);

impl Iterator for IntoIter {
    type Item = Rc<Env>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.take();

        if let Some(rc) = &ret {
            self.0 = rc.parent();
        }

        ret
    }
}
