#![feature(const_vec_new)]

extern crate failure_derive;
extern crate quicli;
extern crate structopt;

use std::str::FromStr;

use quicli::prelude::*;
use structopt::StructOpt;

#[allow(dead_code)]
const NULL: SExp = SExp::List(Vec::new());

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, Fail)]
enum LispError {
    #[fail(display = "could not parse expression: {}", exp)]
    SyntaxError { exp: String },
}

fn is_atom_char(c: char) -> bool {
    !c.is_whitespace() && !c.is_control() && c != '(' && c != ')'
}

fn is_symbol_char(c: char) -> bool {
    is_atom_char(c) && (c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '*')
}

fn find_closing_delim(s: &str, d_plus: char, d_minus: char) -> Option<usize> {
    let mut depth = 0;

    for (idx, c) in s.chars().enumerate() {
        if d_plus == d_minus {
            if c == d_plus {
                depth = !depth;
            }
        } else {
            match c {
                x if x == d_plus => depth += 1,
                x if x == d_minus => depth -= 1,
                _ => (),
            }
        }

        if depth == 0 {
            return Some(idx);
        }
    }

    None
}

#[derive(Debug, PartialEq)]
enum Primitive {
    Boolean(bool),
    Character(char),
    Number(f64),
    String(String),
    Symbol(String),
    // Procedure(Box<Fn(SExp) -> SExp>),
}

impl FromStr for Primitive {
    type Err = LispError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#t" => return Ok(Primitive::Boolean(true)),
            "#f" => return Ok(Primitive::Boolean(false)),
            _ => (),
        }

        match s.parse::<f64>() {
            Ok(num) => return Ok(Primitive::Number(num)),
            _ => (),
        }

        if s.len() == 3 && s.starts_with("'") && s.ends_with("'") {
            return Ok(Primitive::Character(s.chars().skip(1).next().unwrap()));
        }

        if s.starts_with('"') && s.ends_with('"') {
            match find_closing_delim(s, '"', '"') {
                Some(idx) if idx + 1 == s.len() => {
                    return Ok(Primitive::String(s.get(1..idx).unwrap().to_string()));
                }
                _ => (),
            }
        }

        if s.chars().all(is_symbol_char) {
            return Ok(Primitive::Symbol(s.to_string()));
        }

        Err(LispError::SyntaxError { exp: s.to_string() })
    }
}

#[derive(Debug, PartialEq)]
enum SExp {
    Atom(Primitive),
    List(Vec<SExp>),
}

impl FromStr for SExp {
    type Err = LispError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code = s.trim().to_owned();

        if code.chars().all(is_atom_char) {
            debug!("Matched atom: {}", code);
            Ok(SExp::Atom(code.parse::<Primitive>()?))
        } else if code.starts_with('(') && code.ends_with(')') {
            match find_closing_delim(&code, '(', ')') {
                Some(idx) => {
                    debug!("Matched list with length {} chars", idx + 1);
                    let mut list_str = code.clone();
                    let mut list_out: Vec<SExp> = Vec::new();

                    if idx + 1 == code.len() {
                        list_str = code.get(1..idx).unwrap().to_string();
                    }

                    while !list_str.is_empty() {
                        debug!(
                            "Processing list string with length {} chars",
                            list_str.len()
                        );

                        if list_str.starts_with('(') {
                            match find_closing_delim(&list_str, '(', ')') {
                                Some(idx2) => {
                                    if idx2 + 1 == list_str.len() {
                                        debug!("Whole string is a single list");
                                        list_out.push(list_str.parse::<SExp>()?);
                                        break;
                                    } else {
                                        debug!("Matched sub-list with length {} chars", idx2 + 1);
                                        let (before, after) = list_str.split_at(idx2 + 1);
                                        list_out.push(before.parse::<SExp>()?);
                                        list_str = after.trim().to_string();
                                    }
                                }
                                None => {
                                    return Err(LispError::SyntaxError {
                                        exp: list_str.to_string(),
                                    });
                                }
                            }
                        } else {
                            if let Ok(prim_val) = list_str.parse::<Primitive>() {
                                list_out.push(SExp::Atom(prim_val));
                                break;
                            }

                            match list_str.find(|c| !is_atom_char(c)) {
                                Some(idx3) => {
                                    debug!(
                                        "Matched atom in first position with length {} chars",
                                        idx3
                                    );
                                    let (first, rest) = list_str.split_at(idx3);
                                    list_out.push(first.parse::<SExp>()?);
                                    list_str = rest.trim().to_string();
                                }
                                None => {
                                    list_out.push(list_str.parse::<SExp>()?);
                                    break;
                                }
                            }
                        }
                    }

                    Ok(SExp::List(list_out))
                }
                None => Err(LispError::SyntaxError { exp: code }),
            }
        } else {
            let prim = code.parse::<Primitive>()?;
            Ok(SExp::Atom(prim))
        }
    }
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("rsch")?;

    info!("Reading source from {}", args.file);

    let code = read_file(&args.file)?;
    match code.parse::<SExp>() {
        Ok(tree) => println!("{:?}", tree),
        Err(error) => error!("{}", error),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SExp::{Atom, List};
    use super::*;

    fn do_parse_and_assert(test_val: &str, expected_val: SExp) {
        let test_parsed = test_val.parse::<SExp>().unwrap();
        assert_eq!(test_parsed, expected_val);
    }

    fn mk_sym(s: &str) -> SExp {
        Atom(Primitive::Symbol(s.to_string()))
    }

    #[test]
    fn parse_empty_list() {
        do_parse_and_assert("()", NULL);
    }

    #[test]
    fn parse_list_of_lists() {
        do_parse_and_assert("(() () ())", List(vec![NULL, NULL, NULL]));
    }

    #[test]
    fn parse_atom() {
        do_parse_and_assert("hello", mk_sym("hello"));
    }

    #[test]
    fn parse_list_of_atoms() {
        do_parse_and_assert(
            "(a bc de fgh ijk l mnop)",
            List(vec![
                mk_sym("a"),
                mk_sym("bc"),
                mk_sym("de"),
                mk_sym("fgh"),
                mk_sym("ijk"),
                mk_sym("l"),
                mk_sym("mnop"),
            ]),
        );
    }

    #[test]
    fn parse_primitive_types() {
        do_parse_and_assert("#f", Atom(Primitive::Boolean(false)));
        do_parse_and_assert("#t", Atom(Primitive::Boolean(true)));
        do_parse_and_assert("0", Atom(Primitive::Number(0_f64)));
        do_parse_and_assert("2.0", Atom(Primitive::Number(2.0)));
        do_parse_and_assert("inf", Atom(Primitive::Number(std::f64::INFINITY)));
        do_parse_and_assert("-inf", Atom(Primitive::Number(std::f64::NEG_INFINITY)));
        do_parse_and_assert("'c'", Atom(Primitive::Character('c')));
        do_parse_and_assert("'''", Atom(Primitive::Character('\'')));
        do_parse_and_assert(
            r#""test string with spaces""#,
            Atom(Primitive::String("test string with spaces".to_string())),
        );
    }

    #[test]
    fn parse_mixed_type_list() {
        do_parse_and_assert(
            "(0 #f () 33.5 \"xyz\" '?' #t \"\" \"   \")",
            List(vec![
                Atom(Primitive::Number(0_f64)),
                Atom(Primitive::Boolean(false)),
                NULL,
                Atom(Primitive::Number(33.5)),
                Atom(Primitive::String("xyz".to_string())),
                Atom(Primitive::Character('?')),
                Atom(Primitive::Boolean(true)),
                Atom(Primitive::String("".to_string())),
                Atom(Primitive::String("   ".to_string())),
            ]),
        );
    }
}
