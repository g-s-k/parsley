#![cfg(test)]

use super::SExp::{self, Atom, Null};
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

macro_rules! assert_eval_eq {
    ( $lhs:expr, $rhs:expr ) => {
        assert_eq!(eval!($lhs).expect("Evaluation failed"), SExp::from($rhs))
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
    let cond = || SExp::make_symbol("cond");
    let else_ = || SExp::make_symbol("else");
    let t = || SExp::from(true);
    let f = || SExp::from(false);

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(cond()).eval(&mut ctx).unwrap(),
        Atom(Primitive::Void)
    );

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((cond(), ((else_(), ('a',)),)))
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('a')
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(SExp::from(('a',)).cons(else_()))
            .cons(SExp::from(('b',)).cons(t()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('b')
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(SExp::from(('a',)).cons(else_()))
            .cons(SExp::from(('b',)).cons(f()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('a')
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(SExp::from(('a',)).cons(else_()))
            .cons(SExp::from(('d',)).cons(t()))
            .cons(SExp::from(('b',)).cons(t()))
            .cons(SExp::from(('c',)).cons(f()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('b')
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(SExp::from(('a',)).cons(else_()))
            .cons(SExp::from(('d',)).cons(f()))
            .cons(SExp::from(('b',)).cons(f()))
            .cons(SExp::from(('c',)).cons(f()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('a')
    );
}

#[test]
fn begin() {
    let begin = || SExp::make_symbol("begin");

    let mut ctx = Context::default();
    assert!(Null.cons(begin()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((begin(), (0, (1,)))).eval(&mut ctx).unwrap(),
        SExp::from(1)
    )
}

#[test]
fn r#let() {
    let x = || SExp::make_symbol("x");
    let y = || SExp::make_symbol("y");
    let let_ = || SExp::make_symbol("let");

    let mut ctx = Context::default();
    assert!(Null.cons(let_()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert!(Null.cons(Null).cons(let_()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(x())
            .cons(Null.cons(SExp::from((3_f64,)).cons(x())))
            .cons(let_())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(3)
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(y())
            .cons(x())
            .cons(
                Null.cons(SExp::from((5_f64,)).cons(y()))
                    .cons(SExp::from((3_f64,)).cons(x()))
            )
            .cons(let_())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(5)
    );
}
