#![feature(const_vec_new)]

extern crate failure_derive;
extern crate quicli;
extern crate structopt;

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use quicli::prelude::*;
use structopt::StructOpt;

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
    #[fail(display = "symbol is not defined: {}", sym)]
    UndefinedSymbol { sym: SExp },
    #[fail(
        display = "too many arguments provided: expected {}, got {}.",
        right_num, n_args
    )]
    TooManyArguments { n_args: usize, right_num: usize },
    #[fail(display = "{} expects at least one argument.", symbol)]
    NoArgumentsProvided { symbol: String },
    #[fail(display = "Expected a list, got {}.", atom)]
    NotAList { atom: SExp },
    #[fail(display = "Expected a pair, got the null list.")]
    NullList,
}

#[derive(Debug, Clone)]
struct Context(Vec<HashMap<String, SExp>>);

impl Context {
    fn new() -> Self {
        let defs = HashMap::new();
        Context(vec![defs])
    }

    #[allow(dead_code)]
    fn push(&self) -> Self {
        let mut copy = self.clone();
        copy.0.push(HashMap::new());
        copy
    }

    #[allow(dead_code)]
    fn get(&self, key: &str) -> Option<SExp> {
        match self.0.iter().rev().find_map(|w| w.get(key)) {
            Some(exp) => Some(exp.clone()),
            _ => None,
        }
    }

    fn define(&mut self, key: &str, value: SExp) {
        let num_frames = self.0.len();
        self.0[num_frames - 1].insert(key.to_string(), value);
    }
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
    Void,
    Undefined,
    Boolean(bool),
    Character(char),
    Number(f64),
    String(String),
    Symbol(String),
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Void => write!(f, "#<void>"),
            Primitive::Undefined => write!(f, "#<undefined>"),
            Primitive::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Primitive::Character(c) => write!(f, "'{}'", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "'{}", s),
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
        } else if code.starts_with("'(") && code.ends_with(')') {
            Ok(SExp::List(vec![
                SExp::make_symbol("quote"),
                code.get(1..).unwrap().parse::<SExp>()?,
            ]))
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
    fn eval(self, mut ctx: Context) -> Result<Self, LispError> {
        match self {
            SExp::Atom(Primitive::Symbol(sym)) => match ctx.get(&sym) {
                None => Err(LispError::UndefinedSymbol {
                    sym: SExp::make_symbol(&sym),
                }),
                Some(exp) => Ok(exp),
            },
            SExp::Atom(_) => Ok(self),
            SExp::List(_) if self.is_null() => Ok(NULL),
            SExp::List(contents) => {
                // handle special functions
                if let Some(result) = SExp::List(contents.clone()).eval_special_form(&mut ctx) {
                    result
                } else {
                    // handle everything else
                    match contents
                        .into_iter()
                        .map(|e| SExp::eval(e, ctx.clone()))
                        .collect()
                    {
                        Ok(list) => SExp::List(list).apply(&ctx),
                        Err(err) => Err(err),
                    }
                }
            }
        }
    }

    fn eval_special_form(self, ctx: &mut Context) -> Option<Result<Self, LispError>> {
        match self {
            SExp::Atom(_) => None,
            SExp::List(_) if self.is_null() => None,
            SExp::List(contents) => match &contents[0] {
                SExp::Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "let" | "lambda" => {
                        // placeholder, will be gone once all are implemented
                        None
                    }
                    "begin" => match contents.len() {
                        1 => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "begin".to_string(),
                        })),
                        _ => contents
                            .into_iter()
                            .skip(1)
                            .map(|e| e.eval(ctx.clone()))
                            .last(),
                    },
                    "define" => match contents.len() {
                        1 => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "define".to_string(),
                        })),
                        2 => Some(Err(LispError::TooManyArguments {
                            n_args: 1,
                            right_num: 2,
                        })),
                        3 => match &contents[1] {
                            SExp::Atom(Primitive::Symbol(sym)) => {
                                ctx.define(&sym, contents[2].clone());
                                Some(Ok(contents[2].clone()))
                            }
                            exp @ _ => Some(Err(LispError::SyntaxError {
                                exp: exp.to_string(),
                            })),
                        },
                        // need to implement functions
                        _ => None,
                    },
                    "cond" => match contents.len() {
                        1 => Some(Ok(SExp::Atom(Primitive::Void))),
                        _ => {
                            let mapped = contents.into_iter().skip(1).map(|e| e.eval(ctx.clone()));
                            match mapped.clone().find(|e| match e {
                                Err(_) => true,
                                Ok(SExp::List(list)) if list[0] != false.as_atom() => true,
                                _ => false,
                            }) {
                                None => mapped.last(),
                                Some(thing) => match thing {
                                    Ok(SExp::List(vals)) => Some(Ok(vals[1].clone())),
                                    _ => None,
                                },
                            }
                        }
                    },
                    "else" => match contents.len() {
                        2 => Some(Ok(contents[1].clone())),
                        n @ _ => Some(Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        })),
                    },
                    "and" => match contents.len() {
                        1 => Some(Ok(true.as_atom())),
                        _ => {
                            let mapped = contents.into_iter().skip(1).map(|e| e.eval(ctx.clone()));
                            match mapped.clone().find(|e| match e {
                                Err(_) => true,
                                Ok(atom) => *atom == false.as_atom(),
                            }) {
                                None => mapped.last(),
                                thing @ Some(_) => thing,
                            }
                        }
                    },
                    "or" => match contents.len() {
                        1 => Some(Ok(false.as_atom())),
                        _ => {
                            let mapped = contents.into_iter().skip(1).map(|e| e.eval(ctx.clone()));
                            match mapped.clone().find(|e| match e {
                                Err(_) => true,
                                Ok(atom) => *atom != false.as_atom(),
                            }) {
                                None => mapped.last(),
                                thing @ Some(_) => thing,
                            }
                        }
                    },
                    "if" => match contents.len() {
                        4 => {
                            if contents[1] == true.as_atom() {
                                Some(contents[2].clone().eval(ctx.clone()))
                            } else {
                                Some(contents[3].clone().eval(ctx.clone()))
                            }
                        }
                        n @ _ => Some(Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 3,
                        })),
                    },
                    "quote" => match contents.len() {
                        1 => Some(Ok(contents[0].clone())),
                        2 => Some(Ok(contents[1].clone())),
                        n @ _ => Some(Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        })),
                    },
                    _ => None,
                },
                _ => None,
            },
        }
    }

    fn apply(self, ctx: &Context) -> Result<Self, LispError> {
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
                    "list" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        _ => Ok(SExp::List(contents[1..].to_vec())),
                    },
                    "null?" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => Ok(contents[1].is_null().as_atom()),
                        n @ _ => Err(LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    s @ _ => Err(LispError::UndefinedSymbol {
                        sym: SExp::make_symbol(s),
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

    fn make_symbol(sym: &str) -> Self {
        SExp::Atom(Primitive::Symbol(sym.to_string()))
    }
}

trait AsAtom {
    fn as_atom(&self) -> SExp;
}

impl AsAtom for bool {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Boolean(*self))
    }
}

impl AsAtom for char {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Character(*self))
    }
}

impl AsAtom for f64 {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::Number(*self))
    }
}

impl AsAtom for str {
    fn as_atom(&self) -> SExp {
        SExp::Atom(Primitive::String(self.to_string()))
    }
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("rsch")?;

    let base_context = Context::new();

    info!("Reading source from {}", args.file);

    let code = read_file(&args.file)?;
    match code.parse::<SExp>() {
        Ok(tree) => println!("{}", tree.eval(base_context).unwrap()),
        Err(error) => error!("{}", error),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SExp::{self, Atom, List};
    use super::*;

    fn do_parse_and_assert(test_val: &str, expected_val: SExp) {
        let test_parsed = test_val.parse::<SExp>().unwrap();
        assert_eq!(test_parsed, expected_val);
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
        do_parse_and_assert("hello", SExp::make_symbol("hello"));
    }

    #[test]
    fn parse_list_of_atoms() {
        do_parse_and_assert(
            "(a bc de fgh ijk l mnop)",
            List(vec![
                SExp::make_symbol("a"),
                SExp::make_symbol("bc"),
                SExp::make_symbol("de"),
                SExp::make_symbol("fgh"),
                SExp::make_symbol("ijk"),
                SExp::make_symbol("l"),
                SExp::make_symbol("mnop"),
            ]),
        );
    }

    #[test]
    fn parse_primitive_types() {
        do_parse_and_assert("#f", false.as_atom());
        do_parse_and_assert("#t", true.as_atom());
        do_parse_and_assert("0", 0_f64.as_atom());
        do_parse_and_assert("2.0", 2.0.as_atom());
        do_parse_and_assert("inf", std::f64::INFINITY.as_atom());
        do_parse_and_assert("-inf", std::f64::NEG_INFINITY.as_atom());
        do_parse_and_assert("'c'", 'c'.as_atom());
        do_parse_and_assert("'''", '\''.as_atom());
        do_parse_and_assert(
            r#""test string with spaces""#,
            "test string with spaces".as_atom(),
        );
    }

    #[test]
    fn parse_mixed_type_list() {
        do_parse_and_assert(
            "(0 #f () 33.5 \"xyz\" '?' #t \"\" \"   \")",
            List(vec![
                0_f64.as_atom(),
                false.as_atom(),
                NULL,
                33.5.as_atom(),
                "xyz".as_atom(),
                '?'.as_atom(),
                true.as_atom(),
                "".as_atom(),
                "   ".as_atom(),
            ]),
        );
    }

    #[test]
    fn parse_quote_syntax() {
        do_parse_and_assert(
            "'(a b c d)",
            List(vec![
                SExp::make_symbol("quote"),
                List(vec![
                    SExp::make_symbol("a"),
                    SExp::make_symbol("b"),
                    SExp::make_symbol("c"),
                    SExp::make_symbol("d"),
                ]),
            ]),
        );
    }

    #[test]
    fn eval_empty_list() {
        let ctx = Context::new();

        assert_eq!(NULL.eval(ctx).unwrap(), NULL);
    }

    #[test]
    fn eval_atom() {
        let ctx = Context::new;
        let sym = || SExp::make_symbol("test");
        let quote = || SExp::make_symbol("quote");

        assert!(sym().eval(ctx()).is_err());
        assert_eq!(List(vec![quote(), sym()]).eval(ctx()).unwrap(), sym())
    }

    #[test]
    fn eval_list_quote() {
        let ctx = Context::new;

        let test_list = vec![SExp::make_symbol("quote"), NULL];
        assert_eq!(
            List(test_list.clone()).eval(ctx()).unwrap(),
            test_list[1].clone()
        );

        let test_list_2 = vec![
            SExp::make_symbol("quote"),
            List(vec![SExp::make_symbol("abc"), SExp::make_symbol("xyz")]),
        ];
        assert_eq!(
            List(test_list_2.clone()).eval(ctx()).unwrap(),
            test_list_2[1].clone()
        );
    }

    #[test]
    fn eval_null_test() {
        let ctx = Context::new;
        let null = || SExp::make_symbol("null?");

        assert_eq!(
            List(vec![null(), SExp::make_symbol("test")])
                .eval(ctx())
                .unwrap(),
            false.as_atom()
        );
        assert_eq!(
            List(vec![null(), NULL]).eval(ctx()).unwrap(),
            true.as_atom()
        );
        assert_eq!(
            List(vec![
                null(),
                List(vec![
                    SExp::make_symbol("quote"),
                    List(vec![false.as_atom(), NULL])
                ])
            ])
            .eval(ctx())
            .unwrap(),
            false.as_atom()
        );
    }

    #[test]
    fn eval_if() {
        let ctx = Context::new;
        let sym_1 = || "one".as_atom();
        let sym_2 = || "two".as_atom();

        assert_eq!(
            List(vec![
                SExp::make_symbol("if"),
                true.as_atom(),
                sym_1(),
                sym_2()
            ])
            .eval(ctx())
            .unwrap(),
            sym_1()
        );

        assert_eq!(
            List(vec![
                SExp::make_symbol("if"),
                false.as_atom(),
                sym_1(),
                sym_2()
            ])
            .eval(ctx())
            .unwrap(),
            sym_2()
        );
    }

    #[test]
    fn eval_and() {
        let ctx = Context::new;

        let and = || SExp::make_symbol("and");

        assert_eq!(List(vec![and()]).eval(ctx()).unwrap(), true.as_atom());

        assert_eq!(
            List(vec![and(), true.as_atom(), true.as_atom()])
                .eval(ctx())
                .unwrap(),
            true.as_atom()
        );

        assert_eq!(
            List(vec![and(), false.as_atom(), true.as_atom()])
                .eval(ctx())
                .unwrap(),
            false.as_atom()
        );

        assert_eq!(
            List(vec![and(), false.as_atom(), false.as_atom()])
                .eval(ctx())
                .unwrap(),
            false.as_atom()
        );

        assert_eq!(
            List(vec![and(), true.as_atom(), 3.0.as_atom()])
                .eval(ctx())
                .unwrap(),
            3.0.as_atom()
        );

        assert_eq!(List(vec![and(), NULL]).eval(ctx()).unwrap(), NULL);

        assert_eq!(
            List(vec![
                and(),
                'a'.as_atom(),
                'b'.as_atom(),
                false.as_atom(),
                'c'.as_atom()
            ])
            .eval(ctx())
            .unwrap(),
            false.as_atom()
        );
    }

    #[test]
    fn eval_or() {
        let ctx = Context::new;
        let or = || SExp::make_symbol("or");

        assert_eq!(List(vec![or()]).eval(ctx()).unwrap(), false.as_atom());

        assert_eq!(
            List(vec![or(), true.as_atom(), true.as_atom()])
                .eval(ctx())
                .unwrap(),
            true.as_atom()
        );

        assert_eq!(
            List(vec![or(), false.as_atom(), true.as_atom()])
                .eval(ctx())
                .unwrap(),
            true.as_atom()
        );

        assert_eq!(
            List(vec![or(), false.as_atom(), false.as_atom()])
                .eval(ctx())
                .unwrap(),
            false.as_atom()
        );

        assert_eq!(
            List(vec![or(), 3.0.as_atom(), true.as_atom()])
                .eval(ctx())
                .unwrap(),
            3.0.as_atom()
        );

        assert_eq!(List(vec![or(), NULL]).eval(ctx()).unwrap(), NULL);

        assert_eq!(
            List(vec![
                or(),
                false.as_atom(),
                'a'.as_atom(),
                'b'.as_atom(),
                'c'.as_atom()
            ])
            .eval(ctx())
            .unwrap(),
            'a'.as_atom()
        );
    }

    #[test]
    fn eval_cond() {
        let ctx = Context::new;
        let cond = || SExp::make_symbol("cond");
        let else_ = || SExp::make_symbol("else");

        assert_eq!(
            List(vec![cond()]).eval(ctx()).unwrap(),
            Atom(Primitive::Void)
        );

        assert_eq!(
            List(vec![cond(), List(vec![else_(), 'a'.as_atom()])])
                .eval(ctx())
                .unwrap(),
            'a'.as_atom()
        );

        assert_eq!(
            List(vec![
                cond(),
                List(vec![true.as_atom(), 'b'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(ctx())
            .unwrap(),
            'b'.as_atom()
        );

        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'b'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(ctx())
            .unwrap(),
            'a'.as_atom()
        );

        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'c'.as_atom()]),
                List(vec![true.as_atom(), 'b'.as_atom()]),
                List(vec![true.as_atom(), 'd'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(ctx())
            .unwrap(),
            'b'.as_atom()
        );
        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'c'.as_atom()]),
                List(vec![false.as_atom(), 'b'.as_atom()]),
                List(vec![false.as_atom(), 'd'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
                .eval(ctx())
                .unwrap(),
            'a'.as_atom()
        );
    }

    #[test]
    fn eval_begin() {
        let ctx = Context::new;
        let begin = || SExp::make_symbol("begin");

        assert!(List(vec![begin()]).eval(ctx()).is_err());

        assert_eq!(
            List(vec![begin(), 0_f64.as_atom(), 1_f64.as_atom()])
                .eval(ctx())
                .unwrap(),
            1_f64.as_atom()
        )
    }
}
