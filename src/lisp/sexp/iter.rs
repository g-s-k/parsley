use std::iter::FromIterator;

use super::SExp::{self, Atom, Null, Pair};

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
    type Item = Self;
    type IntoIter = SExpIterator;

    fn into_iter(self) -> Self::IntoIter {
        SExpIterator { exp: self }
    }
}

pub struct SExpRefIterator<'a> {
    exp: &'a SExp,
}

impl<'a> Iterator for SExpRefIterator<'a> {
    type Item = &'a SExp;

    fn next(&mut self) -> Option<Self::Item> {
        match self.exp {
            Pair { head, tail } => {
                self.exp = &*tail;
                Some(&*head)
            }
            a @ Atom(_) => {
                self.exp = &Null;
                Some(&a)
            }
            _ => None,
        }
    }
}

impl SExp {
    /// Iterate over an S-Expression, by reference.
    ///
    /// # Example
    /// ```
    /// use parsley::SExp::{self, Null};
    ///
    /// assert_eq!(SExp::from(((),)).iter().next().unwrap(), &Null);
    /// ```
    pub fn iter(&self) -> SExpRefIterator {
        SExpRefIterator { exp: self }
    }
}

impl FromIterator<SExp> for SExp {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SExp>,
    {
        let mut exp_out = Null;
        let mut last = &mut exp_out;

        for exp in iter {
            let new_val = Pair {
                head: Box::new(exp),
                tail: Box::new(Null),
            };

            match last {
                Null => {
                    *last = new_val;
                }
                Pair { ref mut tail, .. } => {
                    *tail = Box::new(new_val);
                    last = tail;
                }
                _ => (),
            }
        }

        exp_out
    }
}
