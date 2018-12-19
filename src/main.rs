#![feature(const_vec_new)]

extern crate failure_derive;
extern crate quicli;
extern crate structopt;

use std::fmt;
use std::str::FromStr;

use quicli::prelude::*;
use structopt::StructOpt;

const NULL: SExp = SExp::List(Vec::new());
const TRUE: SExp = SExp::Atom(Primitive::Boolean(true));
const FALSE: SExp = SExp::Atom(Primitive::Boolean(false));

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
    #[fail(display = "procedure is not defined: {}", proc)]
    UndefinedProcedure { proc: SExp },
    #[fail(
        display = "too many arguments provided: expected {}, got {}.",
        right_num, n_args
    )]
    TooManyArguments { n_args: usize, right_num: usize },
    #[fail(display = "Expected a list, got {}.", atom)]
    NotAList { atom: SExp },
    #[fail(display = "Expected a pair, got the null list.")]
    NullList,
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

#[derive(Debug, PartialEq, Clone)]
enum Primitive {
    Boolean(bool),
    Character(char),
    Number(f64),
    String(String),
    Symbol(String),
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Boolean(b) => write!(f, "{}", b),
            Primitive::Character(c) => write!(f, "'{}'", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "{}", s),
        }
    }
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

#[derive(Debug, PartialEq, Clone)]
enum SExp {
    Atom(Primitive),
    List(Vec<SExp>),
}

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SExp::Atom(a) => write!(f, "{}", a),
            SExp::List(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
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

impl SExp {
    fn eval(self) -> Result<Self, LispError> {
        match self {
            SExp::Atom(_) => Ok(self),
            SExp::List(ref contents) if contents.len() == 0 => Ok(NULL),
            SExp::List(contents) => match contents.into_iter().map(SExp::eval).collect() {
                Ok(list) => SExp::List(list).apply(),
                Err(err) => Err(err),
            },
        }
    }

    fn apply(self) -> Result<Self, LispError> {
        match self {
            SExp::Atom(_) => Ok(self),
            SExp::List(_) if self.is_null() => Ok(NULL),
            SExp::List(contents) => match &contents[0] {
                SExp::Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "car" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => contents[1].car(),
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "cdr" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => contents[1].cdr(),
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "cons" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        3 => Ok(SExp::cons(contents[1].clone(), contents[2].clone())),
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "quote" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => Ok(contents[1].clone()),
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "null?" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => {
                            if contents[1].is_null() {
                                Ok(TRUE)
                            } else {
                                Ok(FALSE)
                            }
                        }
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    s @ _ => Err(LispError::UndefinedProcedure {
                        proc: SExp::Atom(Primitive::Symbol(s.to_string())),
                    }),
                },
                _ => Ok(SExp::List(contents.clone())),
            },
        }
    }

    fn is_null(&self) -> bool {
        match self {
            SExp::List(ref contents) if contents.len() == 0 => true,
            _ => false,
        }
    }

    fn car(&self) -> Result<SExp, LispError> {
        match self {
            atom @ SExp::Atom(_) => Err(LispError::NotAList { atom: atom.clone() }),
            SExp::List(_) if self.is_null() => Err(LispError::NullList),
            SExp::List(contents) => Ok(contents[0].clone()),
        }
    }

    fn cdr(&self) -> Result<SExp, LispError> {
        match self {
            atom @ SExp::Atom(_) => Err(LispError::NotAList { atom: atom.clone() }),
            SExp::List(_) if self.is_null() => Err(LispError::NullList),
            SExp::List(contents) => Ok(SExp::List(contents[1..].to_vec())),
        }
    }

    fn cons(exp1: Self, exp2: Self) -> Self {
        match exp2 {
            SExp::Atom(_) => SExp::List(vec![exp1, exp2]),
            SExp::List(mut contents) => {
                let mut new_contents = vec![exp1];
                new_contents.append(&mut contents);
                SExp::List(new_contents)
            }
        }
    }
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("rsch")?;

    info!("Reading source from {}", args.file);

    let code = read_file(&args.file)?;
    match code.parse::<SExp>() {
        Ok(tree) => println!("{}", tree.eval().unwrap()),
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
        do_parse_and_assert("#f", FALSE);
        do_parse_and_assert("#t", TRUE);
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
                FALSE,
                NULL,
                Atom(Primitive::Number(33.5)),
                Atom(Primitive::String("xyz".to_string())),
                Atom(Primitive::Character('?')),
                TRUE,
                Atom(Primitive::String("".to_string())),
                Atom(Primitive::String("   ".to_string())),
            ]),
        );
    }

    #[test]
    fn eval_empty_list() {
        assert_eq!(NULL.eval().unwrap(), NULL);
    }

    #[test]
    fn eval_atom() {
        let sym = mk_sym("test");
        assert_eq!(sym.clone().eval().unwrap(), sym);
    }

    #[test]
    fn eval_list_quote() {
        let test_list = vec![mk_sym("quote"), NULL];
        assert_eq!(
            List(test_list.clone()).eval().unwrap(),
            test_list[1].clone()
        );
    }

    #[test]
    fn eval_null_test() {
        assert_eq!(
            List(vec![mk_sym("null?"), mk_sym("test")]).eval().unwrap(),
            FALSE
        );
        assert_eq!(List(vec![mk_sym("null?"), NULL]).eval().unwrap(), TRUE);
        assert_eq!(
            List(vec![
                mk_sym("null?"),
                List(vec![mk_sym("quote"), List(vec![FALSE, NULL])])
            ])
            .eval()
            .unwrap(),
            FALSE
        );
    }
}
