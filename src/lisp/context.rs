use std::collections::HashMap;

use super::Primitive::Procedure;
use super::SExp::Atom;
use super::*;

#[derive(Debug, Clone)]
pub struct Context(Vec<HashMap<String, SExp>>);

impl Context {
    pub fn new() -> Self {
        let defs = HashMap::new();
        Context(vec![defs])
    }

    pub fn push(&self) -> Self {
        let mut copy = self.clone();
        copy.0.push(HashMap::new());
        copy
    }

    pub fn get(&self, key: &str) -> Option<SExp> {
        match self.0.iter().rev().find_map(|w| w.get(key)) {
            Some(exp) => Some(exp.clone()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: SExp) {
        for frame in self.0.iter_mut().rev() {
            if frame.contains_key(key) {
                frame.insert(key.to_string(), value);
                break;
            }
        }
    }

    pub fn define(&mut self, key: &str, value: SExp) {
        let num_frames = self.0.len();
        self.0[num_frames - 1].insert(key.to_string(), value);
    }

    pub fn base() -> Self {
        let mut ret = Self::new();

        ret.define(
            "eq?",
            Atom(Procedure(Rc::new(|v| (v[0] == v[1]).as_atom()))),
        );

        ret.define(
            "null?",
            Atom(Procedure(Rc::new(|v| v[0].is_null().as_atom()))),
        );

        ret.define(
            "cons",
            Atom(Procedure(Rc::new(|v| {
                SExp::cons(v[0].to_owned(), v[1].to_owned())
            }))),
        );

        ret
    }
}
