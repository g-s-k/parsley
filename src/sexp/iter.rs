use std::iter::FromIterator;
use std::ops::Index;

use super::SExp::{self, Atom, Null, Pair, Vector};

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
    exp: SExpRef<'a>,
}

enum SExpRef<'a> {
    List(&'a SExp),
    Vect(&'a [SExp]),
}

impl<'a> Iterator for SExpRefIterator<'a> {
    type Item = &'a SExp;

    fn next(&mut self) -> Option<Self::Item> {
        use SExpRef::*;
        match self.exp {
            List(exp) => match exp {
                Pair { head, tail } => {
                    self.exp = List(&*tail);
                    Some(&*head)
                }
                a @ Atom(_) => {
                    self.exp = List(&Null);
                    Some(&a)
                }
                _ => None,
            },
            Vect(v) => {
                self.exp = Vect(&v[1..]);
                Some(&v[0])
            }
        }
    }
}

impl SExp {
    /// Iterate over an S-Expression, by reference.
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// assert_eq!(
    ///     sexp![()].iter().next().unwrap(),
    ///     &SExp::Null
    /// );
    /// ```
    pub fn iter(&self) -> SExpRefIterator {
        if let Vector(v) = self {
            SExpRefIterator {
                exp: SExpRef::Vect(&v),
            }
        } else {
            SExpRefIterator {
                exp: SExpRef::List(self),
            }
        }
    }

    /// Easy way to check for `Null` if you're planning on iterating
    pub fn is_empty(&self) -> bool {
        if let Null = self {
            true
        } else {
            false
        }
    }

    /// Get the length of an S-Expression (vector or list)
    ///
    /// # Example
    /// ```
    /// use parsley::prelude::*;
    /// assert_eq!(
    ///     sexp!['a', "bee", SExp::sym("sea")].len(),
    ///     3
    /// );
    /// ```
    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

impl Index<usize> for SExp {
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        self.iter().nth(index).unwrap()
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
