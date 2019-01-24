use std::str::FromStr;

use super::SExp::{self, Atom, Null, Pair};
use super::{utils, Error, Primitive, Result};

mod tests;

impl FromStr for SExp {
    type Err = Error;

    fn from_str(s: &str) -> Result {
        let trimmed_str = s.trim();

        // TODO: cover more cases where the string contains multiple expressions
        if trimmed_str.starts_with('(') {
            if let Some(idx) = utils::find_closing_delim(trimmed_str.chars(), '(', ')') {
                if idx + 1 < trimmed_str.len() {
                    let fixed_str = format!("(begin {})", trimmed_str);
                    return Self::parse_str(&fixed_str);
                }
            }
        }

        Self::parse_str(trimmed_str)
    }
}

impl SExp {
    fn parse_str(s: &str) -> Result {
        let mut code = s.trim();

        if code.starts_with(';') {
            if let Some(new_idx) = code.find('\n') {
                code = &code[new_idx..].trim();
            } else {
                code = "";
            }
        }

        if code.is_empty() {
            return Ok(Atom(Primitive::Void));
        }

        if code.starts_with('\'') {
            if code.len() == 1 {
                Err(Error::Syntax {
                    exp: code.to_string(),
                })
            } else {
                Ok(Null
                    .cons(Self::parse_str(&code[1..])?)
                    .cons(Self::sym("quote")))
            }
        } else if code.chars().all(utils::is_atom_char) {
            Ok(Atom(code.parse::<Primitive>()?))
        } else if code.starts_with('(') && code.ends_with(')') {
            match utils::find_closing_delim(code.chars(), '(', ')') {
                Some(idx) if idx == 1 => Ok(Null),
                Some(idx) => Self::parse_list_from_str(&code[1..idx]),
                None => Err(Error::Syntax {
                    exp: code.to_string(),
                }),
            }
        } else if code.starts_with("#(") && code.ends_with(')') {
            match utils::find_closing_delim(code[1..].chars(), '(', ')') {
                Some(idx) if idx == 1 => Ok(Atom(Primitive::Vector(Vec::new()))),
                Some(idx) => Ok(Atom(Primitive::Vector(
                    Self::parse_list_from_str(&code[2..=idx])?
                        .into_iter()
                        .collect::<Vec<_>>(),
                ))),
                None => Err(Error::Syntax {
                    exp: code.to_string(),
                }),
            }
        } else {
            let prim = code.parse::<Primitive>()?;
            Ok(Atom(prim))
        }
    }

    fn parse_list_from_str(s: &str) -> Result {
        let mut list_str = s.trim();
        let mut list_out = Null;

        while !list_str.is_empty() {
            println!("{}", list_str);
            match (
                list_str.find(';'),
                list_str.rfind(';'),
                list_str.rfind('\n'),
            ) {
                (Some(_), Some(last), Some(n)) if n < last => {
                    let first_in_line = list_str[n..].find(';').unwrap();
                    list_str = &list_str[..n + first_in_line].trim();
                }
                (Some(first), Some(_), None) => {
                    list_str = &list_str[..first].trim();
                }
                _ => (),
            }

            println!("{}\n", list_str);

            if list_str.ends_with(')') {
                match utils::find_closing_delim(list_str.chars().rev(), ')', '(') {
                    None => {
                        return Err(Error::Syntax {
                            exp: list_str.to_string(),
                        });
                    }
                    Some(idx2) => {
                        let mut new_idx = list_str.len() - 1 - idx2;
                        if new_idx > 0 {
                            // vector
                            if let Some('#') = list_str.chars().nth(new_idx - 1) {
                                new_idx -= 1;
                            }
                            // quotes
                            while let Some('\'') = list_str.chars().nth(new_idx - 1) {
                                new_idx -= 1;
                            }
                        }

                        let (before, after) = list_str.split_at(new_idx);
                        list_str = before.trim();
                        list_out = Pair {
                            head: Box::new(Self::parse_str(after)?),
                            tail: Box::new(list_out),
                        };
                    }
                }
            } else {
                if list_str.ends_with('"') {
                    if let Some(idx2) = list_str.chars().rev().skip(1).position(|e| e == '"') {
                        let (rest, last) = list_str.split_at(list_str.len() - 2 - idx2);
                        list_out = list_out.cons(Atom(last.parse::<Primitive>()?));
                        list_str = rest.trim();
                    } else {
                        return Err(Error::Syntax {
                            exp: list_str.to_string(),
                        });
                    }
                }

                if let Some(idx3) = list_str.chars().rev().position(|c| !utils::is_atom_char(c)) {
                    let (rest, last) = list_str.split_at(list_str.len() - idx3);
                    list_out = Pair {
                        head: Box::new(Self::parse_str(last)?),
                        tail: Box::new(list_out),
                    };
                    list_str = rest.trim();
                } else {
                    list_out = Pair {
                        head: Box::new(Self::parse_str(list_str)?),
                        tail: Box::new(list_out),
                    };
                    break;
                }
            }
        }

        Ok(list_out)
    }
}
