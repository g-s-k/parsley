use std::iter::FromIterator;

use super::SExp::{self, *};

/// An iterator over an S-Expression. Returns list elements until the end of a chain of pairs.
pub struct SExpIterator {
    exp: SExp,
}

impl Iterator for SExpIterator {
    type Item = SExp;

    fn next(&mut self) -> Option<Self::Item> {
        match self.exp.to_owned() {
            Pair { head, tail } => {
                self.exp = *tail;
                Some(*head)
            }
            a @ Atom(_) => {
                self.exp = Null;
                Some(a)
            }
            _ => None,
        }
    }
}

impl IntoIterator for SExp {
    type Item = SExp;
    type IntoIter = SExpIterator;

    fn into_iter(self) -> Self::IntoIter {
        SExpIterator { exp: self }
    }
}

impl FromIterator<SExp> for SExp {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SExp>,
    {
        let mut exp_out = Null;

        let iter_rev = iter.into_iter().collect::<Vec<_>>().into_iter().rev();

        for item in iter_rev {
            exp_out = exp_out.cons(item);
        }

        exp_out
    }
}
