use std::str::FromStr;

use super::SExp::{self, *};
use super::{utils, LispError, LispResult, Primitive};

mod tests;

impl FromStr for SExp {
    type Err = LispError;

    fn from_str(s: &str) -> LispResult {
        let trimmed_str = s.trim();

        if trimmed_str.starts_with('(') {
            if let Some(idx) = utils::find_closing_delim(trimmed_str.chars(), '(', ')') {
                if idx + 1 < trimmed_str.len() {
                    let fixed_str = format!("(begin {})", trimmed_str);
                    return SExp::parse_str(&fixed_str);
                }
            }
        }

        SExp::parse_str(trimmed_str)
    }
}

impl SExp {
    fn parse_str(s: &str) -> LispResult {
        let code = s.trim();

        if code.starts_with('\'')
            && code.len() > 1
            && code.chars().skip(1).all(utils::is_symbol_char)
        {
            debug!("Matched quoted symbol: {}", code);
            Ok(Null
                .cons(Atom(code[1..].parse::<Primitive>()?))
                .cons(SExp::make_symbol("quote")))
        } else if code.chars().all(utils::is_atom_char) {
            debug!("Matched atom: {}", code);
            Ok(Atom(code.parse::<Primitive>()?))
        } else if code.starts_with("'(") && code.ends_with(')') {
            let tail = Box::new(if code.len() == 3 {
                Null.cons(Null)
            } else {
                SExp::parse_str(&code[1..])?
            });
            Ok(tail.cons(SExp::make_symbol("quote")))
        } else if code.starts_with('(') && code.ends_with(')') {
            match utils::find_closing_delim(code.chars(), '(', ')') {
                Some(idx) if idx == 1 => Ok(Null),
                Some(idx) => {
                    debug!("Matched list with length {} chars", idx + 1);
                    let mut list_str = code[1..idx].trim();
                    let mut list_out = Null;

                    while !list_str.is_empty() {
                        debug!(
                            "Processing list string with length {} chars",
                            list_str.len()
                        );

                        if list_str.ends_with(')') {
                            match utils::find_closing_delim(list_str.chars().rev(), ')', '(') {
                                None => {
                                    return Err(LispError::SyntaxError {
                                        exp: list_str.to_string(),
                                    });
                                }
                                Some(idx2) => {
                                    debug!("Matched sub-list with length {} chars", idx2 + 1);
                                    let mut new_idx = list_str.len() - 1 - idx2;
                                    if new_idx > 0 {
                                        if let Some('\'') = list_str.chars().nth(new_idx - 1) {
                                            new_idx -= 1;
                                        }
                                    }

                                    let (before, after) = list_str.split_at(new_idx);
                                    list_str = before.trim();
                                    list_out = Pair {
                                        head: Box::new(SExp::parse_str(after)?),
                                        tail: Box::new(list_out),
                                    };
                                }
                            }
                        } else {
                            if list_str.ends_with('"') {
                                match list_str.chars().rev().skip(1).position(|e| e == '"') {
                                    Some(idx2) => {
                                        debug!("Matched string with length {} chars", idx2);
                                        let (rest, last) =
                                            list_str.split_at(list_str.len() - 2 - idx2);
                                        list_out = list_out.cons(Atom(last.parse::<Primitive>()?));
                                        list_str = rest.trim();
                                    }
                                    None => {
                                        return Err(LispError::SyntaxError {
                                            exp: list_str.to_string(),
                                        });
                                    }
                                }
                            }

                            match list_str.chars().rev().position(|c| !utils::is_atom_char(c)) {
                                Some(idx3) => {
                                    debug!(
                                        "Matched atom in first position with length {} chars",
                                        idx3
                                    );
                                    let (rest, last) = list_str.split_at(list_str.len() - idx3);
                                    list_out = Pair {
                                        head: Box::new(SExp::parse_str(last)?),
                                        tail: Box::new(list_out),
                                    };
                                    list_str = rest.trim();
                                }
                                _ => {
                                    debug!("Entire string is an atom.");
                                    list_out = Pair {
                                        head: Box::new(SExp::parse_str(list_str)?),
                                        tail: Box::new(list_out),
                                    };
                                    break;
                                }
                            }
                        }
                    }

                    Ok(list_out)
                }
                None => Err(LispError::SyntaxError {
                    exp: code.to_string(),
                }),
            }
        } else {
            let prim = code.parse::<Primitive>()?;
            Ok(Atom(prim))
        }
    }
}
