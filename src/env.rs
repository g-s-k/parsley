use std::collections::HashMap;
use std::iter::IntoIterator;
use std::rc::Rc;

use super::SExp;

/// A type to represent an execution environment.
pub type Ns = HashMap<String, SExp>;

type Link = Option<Rc<Env>>;

#[derive(Default)]
pub struct Env {
    env: Ns,
    parent: Link,
}

impl Env {
    pub fn new(parent: Link) -> Self {
        Self {
            parent,
            ..Self::default()
        }
    }

    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn iter(&self) -> Iter {
        Iter { env: Some(self) }
    }

    pub fn get(&self, key: &str) -> Option<SExp> {
        for ns in self.iter() {
            if let Some(val) = ns.env.get(key) {
                return Some(val.clone());
            }
        }

        None
    }
}

pub struct Iter<'a> {
    env: Option<&'a Env>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Env;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.env.take();

        if let Some(rc) = &ret {
            if let Some(r) = &rc.parent {
                self.env = Some(&r);
            }
        }

        ret
    }
}

impl IntoIterator for Env {
    type Item = Rc<Self>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { env: Some(self.into_rc()) }
    }
}

pub struct IntoIter {
    env: Link,
}

impl Iterator for IntoIter {
    type Item = Rc<Env>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.env.take();

        if let Some(rc) = &ret {
            self.env = rc.parent.clone();
        }

        ret
    }
}
