#![cfg(test)]

use super::SExp::{self, Null};
use super::*;

macro_rules! sym {
    ( $name:expr ) => {
        SExp::make_symbol($name)
    };
}

macro_rules! eval {
    ( $e:expr ) => {
        $e.eval(&mut Context::default())
    };
}

macro_rules! eval_base {
    ( $e:expr ) => {
        $e.eval(&mut Context::base())
    };
}

macro_rules! assert_eval_eq {
    ( $lhs:expr, $rhs:expr ) => {
        assert_eq!(eval!($lhs).expect("Evaluation failed"), SExp::from($rhs))
    };
}

macro_rules! assert_eval_base_eq {
    ( $lhs:expr, $rhs:expr ) => {
        assert_eq!(
            eval_base!($lhs).expect("Evaluation failed"),
            SExp::from($rhs)
        )
    };
}

#[test]
fn null_list() {
    assert!(eval!(Null).is_err());
    assert_eval_eq!(sexp![sym!("quote"), Null], Null);
}

#[test]
fn atom() {
    assert!(eval!(sym!("test")).is_err());
    assert_eval_eq!(sexp![sym!("quote"), sym!("test")], sym!("test"))
}

#[test]
fn list_quote() {
    assert_eval_eq!(sexp![sym!("quote"), Null], Null);

    let list = sexp![sym!("abc"), sym!("xyz")];
    assert_eval_eq!(sexp![sym!("quote"), list.clone()], list);
}

#[test]
fn r#if() {
    // ensure the right consequent is returned
    assert_eval_eq!(sexp![sym!("if"), true, "one", "two"], "one");
    assert_eval_eq!(sexp![sym!("if"), false, "one", "two"], "two");
    // ensure only the correct consequent is evaluated
    assert!(eval!(sexp![sym!("if"), true, 4, sym!("potato")]).is_ok());
    assert!(eval!(sexp![sym!("if"), true, sym!("potato"), 5]).is_err());
    assert!(eval!(sexp![sym!("if"), false, 3, sym!("potato")]).is_err());
    assert!(eval!(sexp![sym!("if"), false, sym!("potato"), "hooray"]).is_ok());
}

#[test]
fn and() {
    // validate return value
    assert_eval_eq!(sexp![sym!("and")], true);
    assert_eval_eq!(sexp![sym!("and"), true, true], true);
    assert_eval_eq!(sexp![sym!("and"), false, true], false);
    assert_eval_eq!(sexp![sym!("and"), false, false], false);
    assert_eval_eq!(sexp![sym!("and"), true, 3], 3);
    assert_eval_eq!(sexp![sym!("and"), sexp![sym!("quote"), ()]], ());
    assert_eval_eq!(sexp![sym!("and"), 'a', 'b', false, 'c'], false);
    // ensure that evaluation occurs until a false is encountered
    assert!(eval!(sexp![sym!("and"), false, sym!("potato")]).is_ok());
    assert!(eval!(sexp![sym!("and"), true, sym!("potato")]).is_err());
}

#[test]
fn or() {
    // validate return value
    assert_eval_eq!(sexp![sym!("or")], false);
    assert_eval_eq!(sexp![sym!("or"), true, true], true);
    assert_eval_eq!(sexp![sym!("or"), false, true], true);
    assert_eval_eq!(sexp![sym!("or"), true, false], true);
    assert_eval_eq!(sexp![sym!("or"), false, false], false);
    assert_eval_eq!(sexp![sym!("or"), 3, true], 3);
    assert_eval_eq!(sexp![sym!("or"), sexp![sym!("quote"), ()], 5], ());
    assert_eval_eq!(sexp![sym!("or"), false, 'a', 'b', 'c'], 'a');
    // ensure that evaluation stops at first non-false value
    assert!(eval!(sexp![sym!("or"), false, sym!("potato")]).is_err());
    assert!(eval!(sexp![sym!("or"), true, sym!("potato")]).is_ok());
}

#[test]
fn cond() {
    // validate empty value
    assert_eval_eq!(sexp![sym!("cond")], Primitive::Void);
    // validate else behavior
    assert_eval_eq!(sexp![sym!("cond"), sexp![sym!("else"), 'a']], 'a');
    // validate typical use cases
    assert_eval_eq!(
        sexp![sym!("cond"), sexp![true, 'b'], sexp![sym!("else"), 'a']],
        'b'
    );
    assert_eval_eq!(
        sexp![sym!("cond"), sexp![false, 'b'], sexp![sym!("else"), 'a']],
        'a'
    );
    assert_eval_eq!(
        sexp![
            sym!("cond"),
            sexp![false, 'c'],
            sexp![true, 'b'],
            sexp![true, 'd'],
            sexp![sym!("else"), 'a']
        ],
        'b'
    );
    assert_eval_eq!(
        sexp![
            sym!("cond"),
            sexp![false, 'c'],
            sexp![false, 'b'],
            sexp![false, 'd'],
            sexp![sym!("else"), 'a']
        ],
        'a'
    );
    // ensure that evaluation stops at first non-#f predicate
    assert!(eval!(sexp![
        sym!("cond"),
        sexp![true, sym!("potato")],
        sexp![sym!("else"), "good"]
    ])
    .is_err());
    assert!(eval!(sexp![
        sym!("cond"),
        sexp![true, "good"],
        sexp![sym!("else"), sym!("potato")]
    ])
    .is_ok());
}

#[test]
fn begin() {
    assert!(eval!(sexp![sym!("begin")]).is_err());
    assert_eval_eq!(sexp![sym!("begin"), 0, 1], 1);
}

#[test]
fn r#let() {
    // validate errors for insufficient arguments
    assert!(eval!(sexp![sym!("let")]).is_err());
    assert!(eval!(sexp![sym!("let"), ()]).is_err());
    // very basic case
    assert_eval_eq!(sexp![sym!("let"), sexp![sexp![sym!("x"), 3]], sym!("x")], 3);
    // multiple body statements
    assert_eval_eq!(
        sexp![
            sym!("let"),
            sexp![sexp![sym!("x"), 3], sexp![sym!("y"), 5]],
            sym!("x"),
            sym!("y")
        ],
        5
    );
}

#[test]
fn define() {
    // validate errors for insufficient/too many arguments
    assert!(eval!(sexp![sym!("define")]).is_err());
    assert!(eval!(sexp![sym!("define"), sym!("x")]).is_err());
    assert!(eval!(sexp![sym!("define"), sym!("x"), 3, 7]).is_err());
    // very basic case
    assert_eval_eq!(sexp![sym!("define"), sym!("x"), 3], Primitive::Undefined);
    assert_eval_eq!(
        sexp![
            sym!("begin"),
            sexp![sym!("define"), sym!("x"), 3],
            sym!("x")
        ],
        3
    );
    // functional form
    assert_eval_eq!(
        sexp![
            sym!("define"),
            sexp![sym!("x"), sym!("y")],
            sexp![sym!("+"), 3, sym!("y")]
        ],
        Primitive::Undefined
    );
    assert_eval_eq!(
        sexp![
            sym!("begin"),
            sexp![
                sym!("define"),
                sexp![sym!("x"), sym!("y"), sym!("z")],
                sym!("y")
            ],
            sexp![sym!("x"), 4, 5]
        ],
        4
    );
    assert_eval_eq!(
        sexp![
            sym!("begin"),
            sexp![
                sym!("define"),
                sexp![sym!("x"), sym!("y"), sym!("z")],
                sym!("y"),
                sym!("z")
            ],
            sexp![sym!("x"), 4, 5]
        ],
        5
    );
}

#[test]
fn lambda() {
    // validate argument handling
    assert!(eval!(sexp![sym!("lambda")]).is_err());
    assert!(eval!(sexp![sym!("lambda"), sym!("x")]).is_err());
    assert!(eval!(sexp![sym!("lambda"), sexp![sym!("x")]]).is_err());
    assert!(eval!(sexp![
        sexp![sym!("lambda"), sexp![sym!("x")], sym!("x")],
        3,
        4
    ])
    .is_err());
    // validate behavior
    assert_eval_eq!(
        sexp![sexp![sym!("lambda"), sexp![sym!("x")], sym!("x")], 27],
        27
    );
    assert_eval_eq!(
        sexp![
            sexp![
                sym!("lambda"),
                sexp![sym!("x"), sym!("y")],
                "potato",
                sym!("y"),
                true,
                sym!("x")
            ],
            27,
            35
        ],
        27
    );
    assert_eval_base_eq!(
        sexp![
            sexp![
                sym!("lambda"),
                sexp![sym!("x")],
                sexp![sym!("*"), sym!("x"), sym!("x")]
            ],
            11
        ],
        121
    );
}
