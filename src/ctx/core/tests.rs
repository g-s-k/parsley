#![cfg(test)]

use super::SExp::{self, Null};
use super::*;

fn s(n: &str) -> SExp {
    SExp::sym(n)
}

fn eval(e: SExp) -> Result {
    Context::base().eval(e)
}

macro_rules! assert_eval_eq {
    ( $lhs:expr, $rhs:expr ) => {
        assert_eq!(eval($lhs).expect("Evaluation failed"), SExp::from($rhs))
    };
}

#[test]
fn null_list() {
    assert!(eval(Null).is_err());
    assert_eval_eq!(sexp![s("quote"), Null], Null);
}

#[test]
fn atom() {
    assert!(eval(s("test")).is_err());
    assert_eval_eq!(sexp![s("quote"), s("test")], s("test"))
}

#[test]
fn quote() {
    assert_eval_eq!(sexp![s("quote"), Null], Null);

    let list = sexp![s("abc"), s("xyz")];
    assert_eval_eq!(sexp![s("quote"), list.clone()], list);
}

#[test]
fn quasiquote() {
    assert_eval_eq!(sexp![s("quasiquote"), Null], Null);

    let list = sexp![1, 2, false, "foobar"];
    assert_eval_eq!(sexp![s("quasiquote"), list.clone()], list);

    assert_eval_eq!(
        sexp![
            s("quasiquote"),
            sexp![
                'a',
                "hello world",
                sexp![s("unquote"), sexp![s("null?"), s("null")]],
                6
            ]
        ],
        sexp!['a', "hello world", true, 6]
    );
}

#[test]
fn r#if() {
    // ensure the right consequent is returned
    assert_eval_eq!(sexp![s("if"), true, "one", "two"], "one");
    assert_eval_eq!(sexp![s("if"), false, "one", "two"], "two");
    // ensure only the correct consequent is evaluated
    assert!(eval(sexp![s("if"), true, 4, s("potato")]).is_ok());
    assert!(eval(sexp![s("if"), true, s("potato"), 5]).is_err());
    assert!(eval(sexp![s("if"), false, 3, s("potato")]).is_err());
    assert!(eval(sexp![s("if"), false, s("potato"), "hooray"]).is_ok());
}

#[test]
fn and() {
    // validate return value
    assert_eval_eq!(sexp![s("and")], true);
    assert_eval_eq!(sexp![s("and"), true, true], true);
    assert_eval_eq!(sexp![s("and"), false, true], false);
    assert_eval_eq!(sexp![s("and"), false, false], false);
    assert_eval_eq!(sexp![s("and"), true, 3], 3);
    assert_eval_eq!(sexp![s("and"), sexp![s("quote"), ()]], ());
    assert_eval_eq!(sexp![s("and"), 'a', 'b', false, 'c'], false);
    // ensure that evaluation occurs until a false is encountered
    assert!(eval(sexp![s("and"), false, s("potato")]).is_ok());
    assert!(eval(sexp![s("and"), true, s("potato")]).is_err());
}

#[test]
fn or() {
    // validate return value
    assert_eval_eq!(sexp![s("or")], false);
    assert_eval_eq!(sexp![s("or"), true, true], true);
    assert_eval_eq!(sexp![s("or"), false, true], true);
    assert_eval_eq!(sexp![s("or"), true, false], true);
    assert_eval_eq!(sexp![s("or"), false, false], false);
    assert_eval_eq!(sexp![s("or"), 3, true], 3);
    assert_eval_eq!(sexp![s("or"), sexp![s("quote"), ()], 5], ());
    assert_eval_eq!(sexp![s("or"), false, 'a', 'b', 'c'], 'a');
    // ensure that evaluation stops at first non-false value
    assert!(eval(sexp![s("or"), false, s("potato")]).is_err());
    assert!(eval(sexp![s("or"), true, s("potato")]).is_ok());
}

#[test]
fn cond() {
    // validate empty value
    assert_eval_eq!(sexp![s("cond")], Primitive::Void);
    // validate else behavior
    assert_eval_eq!(sexp![s("cond"), sexp![s("else"), 'a']], 'a');
    // validate typical use cases
    assert_eval_eq!(
        sexp![s("cond"), sexp![true, 'b'], sexp![s("else"), 'a']],
        'b'
    );
    assert_eval_eq!(
        sexp![s("cond"), sexp![false, 'b'], sexp![s("else"), 'a']],
        'a'
    );
    assert_eval_eq!(
        sexp![
            s("cond"),
            sexp![false, 'c'],
            sexp![true, 'b'],
            sexp![true, 'd'],
            sexp![s("else"), 'a']
        ],
        'b'
    );
    assert_eval_eq!(
        sexp![
            s("cond"),
            sexp![false, 'c'],
            sexp![false, 'b'],
            sexp![false, 'd'],
            sexp![s("else"), 'a']
        ],
        'a'
    );
    // multiple consequent expressions
    assert_eval_eq!(
        sexp![
            s("cond"),
            sexp![true, 3, sexp![s("null?"), s("null")]],
            sexp![s("else"), 3]
        ],
        true
    );
    // ensure that evaluation stops at first non-#f predicate
    assert!(eval(sexp![
        s("cond"),
        sexp![true, s("potato")],
        sexp![s("else"), "good"]
    ])
    .is_err());
    assert!(eval(sexp![
        s("cond"),
        sexp![true, "good"],
        sexp![s("else"), s("potato")]
    ])
    .is_ok());
}

#[test]
fn begin() {
    assert_eval_eq!(sexp![s("begin")], Primitive::Undefined);
    assert_eval_eq!(sexp![s("begin"), 0, 1], 1);
}

#[test]
fn r#let() {
    // validate errors for insufficient arguments
    assert!(eval(sexp![s("let")]).is_err());
    assert!(eval(sexp![s("let"), ()]).is_err());
    // very basic case
    assert_eval_eq!(sexp![s("let"), sexp![sexp![s("x"), 3]], s("x")], 3);
    // multiple body statements
    assert_eval_eq!(
        sexp![
            s("let"),
            sexp![sexp![s("x"), 3], sexp![s("y"), 5]],
            s("x"),
            s("y")
        ],
        5
    );
}

#[test]
fn define() {
    // validate errors for insufficient/too many arguments
    assert!(eval(sexp![s("define")]).is_err());
    assert!(eval(sexp![s("define"), s("x")]).is_ok());
    assert!(eval(sexp![s("define"), s("x"), 3, 7]).is_err());
    // very basic case
    assert_eval_eq!(sexp![s("define"), s("x"), 3], Primitive::Undefined);
    assert_eval_eq!(sexp![s("begin"), sexp![s("define"), s("x"), 3], s("x")], 3);
    // functional form
    assert_eval_eq!(
        sexp![s("define"), sexp![s("x"), s("y")], sexp![s("+"), 3, s("y")]],
        Primitive::Undefined
    );
    assert_eval_eq!(
        sexp![
            s("begin"),
            sexp![s("define"), sexp![s("x"), s("y"), s("z")], s("y")],
            sexp![s("x"), 4, 5]
        ],
        4
    );
    assert_eval_eq!(
        sexp![
            s("begin"),
            sexp![s("define"), sexp![s("x"), s("y"), s("z")], s("y"), s("z")],
            sexp![s("x"), 4, 5]
        ],
        5
    );
}

#[test]
fn lambda() {
    // validate argument handling
    assert!(eval(sexp![s("lambda")]).is_err());
    assert!(eval(sexp![s("lambda"), s("x")]).is_err());
    assert!(eval(sexp![s("lambda"), sexp![s("x")]]).is_err());
    assert!(eval(sexp![sexp![s("lambda"), sexp![s("x")], s("x")], 3, 4]).is_err());
    // validate behavior
    assert_eval_eq!(sexp![sexp![s("lambda"), sexp![s("x")], s("x")], 27], 27);
    assert_eval_eq!(
        sexp![
            sexp![
                s("lambda"),
                sexp![s("x"), s("y")],
                "potato",
                s("y"),
                true,
                s("x")
            ],
            27,
            35
        ],
        27
    );
    assert_eval_eq!(
        sexp![
            sexp![s("lambda"), sexp![s("x")], sexp![s("*"), s("x"), s("x")]],
            11
        ],
        121
    );
}
