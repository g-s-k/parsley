use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::*;

mod errors;
mod utils;

const NULL: SExp = SExp::List(Vec::new());

#[derive(Debug, Clone)]
pub struct Context(Vec<HashMap<String, SExp>>);

impl Context {
    pub fn new() -> Self {
        let defs = HashMap::new();
        Context(vec![defs])
    }

    fn push(&self) -> Self {
        let mut copy = self.clone();
        copy.0.push(HashMap::new());
        copy
    }

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

pub enum Primitive {
    Void,
    Undefined,
    Boolean(bool),
    Character(char),
    Number(f64),
    String(String),
    Symbol(String),
    Procedure(Box<dyn Fn(Vec<SExp>) -> SExp>),
}

impl Clone for Primitive {
    fn clone(&self) -> Self {
        match self {
            Primitive::Void => Primitive::Void,
            Primitive::Undefined => Primitive::Undefined,
            Primitive::Boolean(b) => Primitive::Boolean(*b),
            Primitive::Character(c) => Primitive::Character(*c),
            Primitive::Number(n) => Primitive::Number(*n),
            Primitive::String(s) => Primitive::String(s.to_string()),
            Primitive::Symbol(s) => Primitive::Symbol(s.to_string()),
            Primitive::Procedure(_) => Primitive::Procedure(Box::new(|_| NULL)),
        }
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Primitive::Void => {
                if let Primitive::Void = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Undefined => {
                if let Primitive::Undefined = other {
                    true
                } else {
                    false
                }
            }
            Primitive::Boolean(b1) => {
                if let Primitive::Boolean(b2) = other {
                    b1 == b2
                } else {
                    false
                }
            }
            Primitive::Character(c1) => {
                if let Primitive::Character(c2) = other {
                    c1 == c2
                } else {
                    false
                }
            }
            Primitive::Number(n1) => {
                if let Primitive::Number(n2) = other {
                    n1 == n2
                } else {
                    false
                }
            }
            Primitive::String(s1) => {
                if let Primitive::String(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Symbol(s1) => {
                if let Primitive::Symbol(s2) = other {
                    s1 == s2
                } else {
                    false
                }
            }
            Primitive::Procedure(_) => false,
        }
    }
}

impl fmt::Debug for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Void => write!(f, "#<void>"),
            Primitive::Undefined => write!(f, "#<undefined>"),
            Primitive::Boolean(b) => write!(f, "<boolean {}>", b),
            Primitive::Character(c) => write!(f, "'{}'", c),
            Primitive::Number(n) => write!(f, "{}", n),
            Primitive::String(s) => write!(f, "\"{}\"", s),
            Primitive::Symbol(s) => write!(f, "'{}", s),
            Primitive::Procedure(_) => write!(f, "#<procedure>"),
        }
    }
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
            Primitive::Procedure(_) => write!(f, "#<procedure>"),
        }
    }
}

impl FromStr for Primitive {
    type Err = errors::LispError;

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
            match utils::find_closing_delim(s, '"', '"') {
                Some(idx) if idx + 1 == s.len() => {
                    return Ok(Primitive::String(s.get(1..idx).unwrap().to_string()));
                }
                _ => (),
            }
        }

        if s.chars().all(utils::is_symbol_char) {
            return Ok(Primitive::Symbol(s.to_string()));
        }

        Err(errors::LispError::SyntaxError { exp: s.to_string() })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SExp {
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
    type Err = errors::LispError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code = s.trim().to_owned();

        if code.chars().all(utils::is_atom_char) {
            debug!("Matched atom: {}", code);
            Ok(SExp::Atom(code.parse::<Primitive>()?))
        } else if code.starts_with("'(") && code.ends_with(')') {
            Ok(SExp::List(vec![
                SExp::make_symbol("quote"),
                code.get(1..).unwrap().parse::<SExp>()?,
            ]))
        } else if code.starts_with('(') && code.ends_with(')') {
            match utils::find_closing_delim(&code, '(', ')') {
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
                            match utils::find_closing_delim(&list_str, '(', ')') {
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
                                    return Err(errors::LispError::SyntaxError {
                                        exp: list_str.to_string(),
                                    });
                                }
                            }
                        } else {
                            if let Ok(prim_val) = list_str.parse::<Primitive>() {
                                list_out.push(SExp::Atom(prim_val));
                                break;
                            }

                            match list_str.find(|c| !utils::is_atom_char(c)) {
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
                None => Err(errors::LispError::SyntaxError { exp: code }),
            }
        } else {
            let prim = code.parse::<Primitive>()?;
            Ok(SExp::Atom(prim))
        }
    }
}

impl SExp {
    pub fn eval(self, ctx: &mut Context) -> Result<Self, errors::LispError> {
        match self {
            SExp::Atom(Primitive::Symbol(sym)) => match ctx.get(&sym) {
                None => Err(errors::LispError::UndefinedSymbol { sym }),
                Some(exp) => Ok(exp),
            },
            SExp::Atom(_) => Ok(self),
            SExp::List(_) if self.is_null() => Ok(NULL),
            SExp::List(contents) => {
                // handle special functions
                if let Some(result) = SExp::List(contents.clone()).eval_special_form(ctx) {
                    debug!("Special form finished evaluating.");
                    result
                } else {
                    // handle everything else
                    debug!("Evaluating normal list.");
                    match contents.into_iter().map(|e| e.eval(ctx)).collect() {
                        Ok(list) => SExp::List(list).apply(ctx),
                        Err(err) => Err(err),
                    }
                }
            }
        }
    }

    fn eval_special_form(self, ctx: &mut Context) -> Option<Result<Self, errors::LispError>> {
        match self {
            SExp::Atom(_) => None,
            SExp::List(_) if self.is_null() => None,
            SExp::List(contents) => match &contents[0] {
                SExp::Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "lambda" => match contents.len() {
                        1 => Some(Err(errors::LispError::NoArgumentsProvided {
                            symbol: "lambda".to_string(),
                        })),
                        2 => Some(Err(errors::LispError::TooManyArguments {
                            n_args: 1,
                            right_num: 2,
                        })),
                        _ => match contents[1].clone() {
                            SExp::List(params) => {
                                debug!("Creating procedure with {} parameters.", params.len());
                                let params_ = params.to_owned();
                                Some(Ok(SExp::Atom(Primitive::Procedure(Box::new(
                                    move |args| {
                                        let mut elems = vec![SExp::make_symbol("let")];
                                        let bound_params = params_
                                            .iter()
                                            .zip(args.into_iter())
                                            .map(|p| SExp::List(vec![p.0.clone(), p.1.clone()]))
                                            .collect::<Vec<_>>();
                                        elems.push(SExp::List(bound_params));
                                        for expr in contents.clone().into_iter().skip(2) {
                                            elems.push(expr);
                                        }
                                        SExp::List(elems)
                                    },
                                )))))
                            }
                            expr @ _ => Some(Err(errors::LispError::SyntaxError {
                                exp: expr.to_string(),
                            })),
                        },
                    },
                    "begin" => match contents.len() {
                        1 => Some(Err(errors::LispError::NoArgumentsProvided {
                            symbol: "begin".to_string(),
                        })),
                        _ => {
                            debug!("Evaluating \"begin\" sequence.");
                            contents.into_iter().skip(1).map(|e| e.eval(ctx)).last()
                        }
                    },
                    "define" => match contents.len() {
                        1 => Some(Err(errors::LispError::NoArgumentsProvided {
                            symbol: "define".to_string(),
                        })),
                        2 => Some(Err(errors::LispError::TooManyArguments {
                            n_args: 1,
                            right_num: 2,
                        })),
                        n_args @ _ => match &contents[1] {
                            SExp::Atom(Primitive::Symbol(sym)) => {
                                if n_args == 3 {
                                    debug!("Defining a quanitity with symbol {}", &sym);
                                    ctx.define(&sym, contents[2].clone());
                                    Some(Ok(contents[2].clone()))
                                } else {
                                    Some(Err(errors::LispError::TooManyArguments {
                                        n_args: n_args - 1,
                                        right_num: 2,
                                    }))
                                }
                            }
                            SExp::List(signature) if signature.len() != 0 => match &signature[0] {
                                SExp::Atom(Primitive::Symbol(sym)) => {
                                    debug!("Defining a function with \"define\" syntax.");
                                    let mut exprs = vec![
                                        SExp::make_symbol("lambda"),
                                        SExp::List(signature[1..].to_vec()),
                                    ];
                                    for expr in contents[2..].into_iter() {
                                        exprs.push(expr.to_owned());
                                    }
                                    ctx.define(&sym, SExp::List(exprs));
                                    Some(Ok(SExp::Atom(Primitive::Undefined)))
                                }
                                exp @ _ => Some(Err(errors::LispError::SyntaxError {
                                    exp: exp.to_string(),
                                })),
                            },
                            exp @ _ => Some(Err(errors::LispError::SyntaxError {
                                exp: exp.to_string(),
                            })),
                        },
                        // need to implement functions
                    },
                    "let" => match contents.len() {
                        1 => Some(Err(errors::LispError::NoArgumentsProvided {
                            symbol: "let".to_string(),
                        })),
                        2 => Some(Err(errors::LispError::TooManyArguments {
                            n_args: 1,
                            right_num: 2,
                        })),
                        _ => match &contents[1] {
                            SExp::List(vals) => {
                                debug!("Creating a local binding.");
                                let mut new_ctx = ctx.push();
                                match vals.iter().find_map(|e| match e {
                                    SExp::List(kv) if kv.len() == 2 => match &kv[0] {
                                        SExp::Atom(Primitive::Symbol(key)) => {
                                            match kv[1].clone().eval(ctx) {
                                                Ok(val) => {
                                                    new_ctx.define(key, val);
                                                    None
                                                }
                                                Err(stuff) => Some(Err(errors::LispError::SyntaxError {
                                                    exp: stuff.to_string(),
                                                })),
                                            }
                                        }
                                        stuff @ _ => Some(Err(errors::LispError::SyntaxError {
                                            exp: stuff.to_string(),
                                        })),
                                    },
                                    stuff @ _ => Some(Err(errors::LispError::SyntaxError {
                                        exp: stuff.to_string(),
                                    })),
                                }) {
                                    None => contents
                                        .into_iter()
                                        .skip(2)
                                        .map(|e| e.eval(&mut new_ctx))
                                        .last(),
                                    stuff @ _ => stuff,
                                }
                            }
                            stuff @ _ => Some(Err(errors::LispError::SyntaxError {
                                exp: stuff.to_string(),
                            })),
                        },
                    },
                    "cond" => match contents.len() {
                        1 => Some(Ok(SExp::Atom(Primitive::Void))),
                        _ => {
                            debug!("Evaluating conditional form.");
                            let false_ = false.as_atom();
                            let else_ = SExp::make_symbol("else");

                            match contents
                                .into_iter()
                                .skip(1)
                                .map(|expr| match expr {
                                    SExp::List(pair) => {
                                        if pair.len() == 2 {
                                            if pair[0] == else_ {
                                                Some(pair[1].clone().eval(ctx))
                                            } else {
                                                match pair[0].clone().eval(ctx) {
                                                    Ok(condition) => {
                                                        if condition == false_ {
                                                            None
                                                        } else {
                                                            Some(pair[1].clone().eval(ctx))
                                                        }
                                                    }
                                                    err @ _ => Some(err),
                                                }
                                            }
                                        } else {
                                            Some(Err(errors::LispError::SyntaxError {
                                                exp: format!("{:?}", pair),
                                            }))
                                        }
                                    }
                                    other @ _ => Some(Err(errors::LispError::SyntaxError {
                                        exp: other.to_string(),
                                    })),
                                })
                                .skip_while(Option::is_none)
                                .next()
                            {
                                Some(stuff) => stuff,
                                None => Some(Err(errors::LispError::SyntaxError {
                                    exp: "malformed 'cond' statement".to_string(),
                                })),
                            }
                        }
                    },
                    "and" => match contents.len() {
                        1 => Some(Ok(true.as_atom())),
                        n @ _ => {
                            debug!("Evaluating 'and' expression.");
                            let false_ = false.as_atom();
                            let mut good_stuff = contents
                                .into_iter()
                                .skip(1)
                                .map(|expr| expr.eval(ctx))
                                .take_while(|element| match element {
                                    Err(_) => false,
                                    Ok(ref atom) if *atom == false_ => false,
                                    _ => true,
                                })
                                .collect::<Vec<_>>();

                            if good_stuff.len() == n - 1 {
                                good_stuff.pop()
                            } else {
                                Some(Ok(false_))
                            }
                        }
                    },
                    "or" => match contents.len() {
                        1 => Some(Ok(false.as_atom())),
                        _ => {
                            debug!("Evaluating 'or' expression.");
                            let false_ = false.as_atom();
                            match contents.into_iter().skip(1).find_map(|expr| {
                                match expr.eval(ctx) {
                                    Ok(ref atom) if *atom == false_ => None,
                                    thing @ _ => Some(thing),
                                }
                            }) {
                                None => Some(Ok(false_)),
                                thing @ _ => thing,
                            }
                        }
                    },
                    "if" => match contents.len() {
                        4 => {
                            debug!("Evaluating 'if' expression.");
                            if contents[1] == true.as_atom() {
                                Some(contents[2].clone().eval(ctx))
                            } else {
                                Some(contents[3].clone().eval(ctx))
                            }
                        }
                        n @ _ => Some(Err(errors::LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 3,
                        })),
                    },
                    "quote" => match contents.len() {
                        2 => Some(Ok(contents[1].clone())),
                        n @ _ => Some(Err(errors::LispError::TooManyArguments {
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

    fn apply(self, ctx: &mut Context) -> Result<Self, errors::LispError> {
        match self {
            SExp::Atom(_) => Ok(self),
            SExp::List(_) if self.is_null() => Ok(NULL),
            SExp::List(contents) => match &contents[0] {
                SExp::Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "car" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => contents[1].car(),
                        n @ _ => Err(errors::LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "cdr" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        2 => contents[1].cdr(),
                        n @ _ => Err(errors::LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    "cons" => match contents.len() {
                        1 => Ok(contents[0].clone()),
                        3 => Ok(SExp::cons(contents[1].clone(), contents[2].clone())),
                        n @ _ => Err(errors::LispError::TooManyArguments {
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
                        n @ _ => Err(errors::LispError::TooManyArguments {
                            n_args: n - 1,
                            right_num: 1,
                        }),
                    },
                    s @ _ => Err(errors::LispError::UndefinedSymbol { sym: s.to_string() }),
                },
                SExp::Atom(Primitive::Procedure(proc)) => {
                    debug!("Applying a procedure.");
                    proc(contents[1..].to_vec()).eval(ctx)
                }
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

    fn car(&self) -> Result<SExp, errors::LispError> {
        match self {
            atom @ SExp::Atom(_) => Err(errors::LispError::NotAList {
                atom: atom.to_string(),
            }),
            SExp::List(_) if self.is_null() => Err(errors::LispError::NullList),
            SExp::List(contents) => Ok(contents[0].clone()),
        }
    }

    fn cdr(&self) -> Result<SExp, errors::LispError> {
        match self {
            atom @ SExp::Atom(_) => Err(errors::LispError::NotAList {
                atom: atom.to_string(),
            }),
            SExp::List(_) if self.is_null() => Err(errors::LispError::NullList),
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
        let mut ctx = Context::new();

        assert_eq!(NULL.eval(&mut ctx).unwrap(), NULL);
    }

    #[test]
    fn eval_atom() {
        let sym = || SExp::make_symbol("test");
        let quote = || SExp::make_symbol("quote");

        let mut ctx = Context::new();
        assert!(sym().eval(&mut ctx).is_err());

        let mut ctx = Context::new();
        assert_eq!(List(vec![quote(), sym()]).eval(&mut ctx).unwrap(), sym())
    }

    #[test]
    fn eval_list_quote() {
        let test_list = vec![SExp::make_symbol("quote"), NULL];
        let mut ctx = Context::new();
        assert_eq!(
            List(test_list.clone()).eval(&mut ctx).unwrap(),
            test_list[1].clone()
        );

        let test_list_2 = vec![
            SExp::make_symbol("quote"),
            List(vec![SExp::make_symbol("abc"), SExp::make_symbol("xyz")]),
        ];
        let mut ctx = Context::new();
        assert_eq!(
            List(test_list_2.clone()).eval(&mut ctx).unwrap(),
            test_list_2[1].clone()
        );
    }

    #[test]
    fn eval_null_test() {
        let null = || SExp::make_symbol("null?");

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![null(), SExp::make_symbol("test")])
                .eval(&mut ctx)
                .unwrap(),
            false.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![null(), NULL]).eval(&mut ctx).unwrap(),
            true.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                null(),
                List(vec![
                    SExp::make_symbol("quote"),
                    List(vec![false.as_atom(), NULL])
                ])
            ])
            .eval(&mut ctx)
            .unwrap(),
            false.as_atom()
        );
    }

    #[test]
    fn eval_if() {
        let sym_1 = || "one".as_atom();
        let sym_2 = || "two".as_atom();

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                SExp::make_symbol("if"),
                true.as_atom(),
                sym_1(),
                sym_2()
            ])
            .eval(&mut ctx)
            .unwrap(),
            sym_1()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                SExp::make_symbol("if"),
                false.as_atom(),
                sym_1(),
                sym_2()
            ])
            .eval(&mut ctx)
            .unwrap(),
            sym_2()
        );
    }

    #[test]
    fn eval_and() {
        let and = || SExp::make_symbol("and");

        let mut ctx = Context::new();
        assert_eq!(List(vec![and()]).eval(&mut ctx).unwrap(), true.as_atom());

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![and(), true.as_atom(), true.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            true.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![and(), false.as_atom(), true.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            false.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![and(), false.as_atom(), false.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            false.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![and(), true.as_atom(), 3.0.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            3.0.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(List(vec![and(), NULL]).eval(&mut ctx).unwrap(), NULL);

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                and(),
                'a'.as_atom(),
                'b'.as_atom(),
                false.as_atom(),
                'c'.as_atom()
            ])
            .eval(&mut ctx)
            .unwrap(),
            false.as_atom()
        );
    }

    #[test]
    fn eval_or() {
        let or = || SExp::make_symbol("or");

        let mut ctx = Context::new();
        assert_eq!(List(vec![or()]).eval(&mut ctx).unwrap(), false.as_atom());

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![or(), true.as_atom(), true.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            true.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![or(), false.as_atom(), true.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            true.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![or(), false.as_atom(), false.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            false.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![or(), 3.0.as_atom(), true.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            3.0.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(List(vec![or(), NULL]).eval(&mut ctx).unwrap(), NULL);

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                or(),
                false.as_atom(),
                'a'.as_atom(),
                'b'.as_atom(),
                'c'.as_atom()
            ])
            .eval(&mut ctx)
            .unwrap(),
            'a'.as_atom()
        );
    }

    #[test]
    fn eval_cond() {
        let cond = || SExp::make_symbol("cond");
        let else_ = || SExp::make_symbol("else");

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![cond()]).eval(&mut ctx).unwrap(),
            Atom(Primitive::Void)
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![cond(), List(vec![else_(), 'a'.as_atom()])])
                .eval(&mut ctx)
                .unwrap(),
            'a'.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                cond(),
                List(vec![true.as_atom(), 'b'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(&mut ctx)
            .unwrap(),
            'b'.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'b'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(&mut ctx)
            .unwrap(),
            'a'.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'c'.as_atom()]),
                List(vec![true.as_atom(), 'b'.as_atom()]),
                List(vec![true.as_atom(), 'd'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(&mut ctx)
            .unwrap(),
            'b'.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                cond(),
                List(vec![false.as_atom(), 'c'.as_atom()]),
                List(vec![false.as_atom(), 'b'.as_atom()]),
                List(vec![false.as_atom(), 'd'.as_atom()]),
                List(vec![else_(), 'a'.as_atom()])
            ])
            .eval(&mut ctx)
            .unwrap(),
            'a'.as_atom()
        );
    }

    #[test]
    fn eval_begin() {
        let begin = || SExp::make_symbol("begin");

        let mut ctx = Context::new();
        assert!(List(vec![begin()]).eval(&mut ctx).is_err());

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![begin(), 0_f64.as_atom(), 1_f64.as_atom()])
                .eval(&mut ctx)
                .unwrap(),
            1_f64.as_atom()
        )
    }

    #[test]
    fn eval_let() {
        let x = || SExp::make_symbol("x");
        let y = || SExp::make_symbol("y");
        let let_ = || SExp::make_symbol("let");

        let mut ctx = Context::new();
        assert!(List(vec![let_()]).eval(&mut ctx).is_err());

        let mut ctx = Context::new();
        assert!(List(vec![let_(), List(vec![])]).eval(&mut ctx).is_err());

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                let_(),
                List(vec![List(vec![x(), 3_f64.as_atom()])]),
                x()
            ])
            .eval(&mut ctx)
            .unwrap(),
            3_f64.as_atom()
        );

        let mut ctx = Context::new();
        assert_eq!(
            List(vec![
                let_(),
                List(vec![
                    List(vec![x(), 3_f64.as_atom()]),
                    List(vec![y(), 5_f64.as_atom()])
                ]),
                x(),
                y()
            ])
            .eval(&mut ctx)
            .unwrap(),
            5_f64.as_atom()
        );
    }
}
