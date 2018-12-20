use std::collections::HashMap;

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

    pub fn define(&mut self, key: &str, value: SExp) {
        let num_frames = self.0.len();
        self.0[num_frames - 1].insert(key.to_string(), value);
    }
}
