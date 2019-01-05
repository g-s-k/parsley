#![cfg(test)]

use super::SExp::{self, Atom, Null, Pair};
use super::*;

#[test]
fn empty_list() {
    let mut ctx = Context::default();
    assert!(Null.eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null)
            .cons(SExp::make_symbol("quote"))
            .eval(&mut ctx)
            .unwrap(),
        Null
    );
}

#[test]
fn atom() {
    let sym = || SExp::make_symbol("test");
    let quote = || SExp::make_symbol("quote");

    let mut ctx = Context::default();
    assert!(sym().eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(sym()).cons(quote()).eval(&mut ctx).unwrap(),
        sym()
    )
}

#[test]
fn list_quote() {
    let test_list = Null.cons(Null).cons(SExp::make_symbol("quote"));
    let mut ctx = Context::default();
    assert_eq!(test_list.eval(&mut ctx).unwrap(), Null);

    let test_list_2 = Null
        .cons(SExp::make_symbol("xyz"))
        .cons(SExp::make_symbol("abc"));
    let test_again = Null
        .cons(test_list_2.clone())
        .cons(SExp::make_symbol("quote"));
    let mut ctx = Context::default();
    assert_eq!(test_again.eval(&mut ctx).unwrap(), test_list_2);
}

#[test]
fn r#if() {
    let sym_1 = || SExp::from("one");
    let sym_2 = || SExp::from("two");

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(sym_2())
            .cons(sym_1())
            .cons(SExp::from(true))
            .cons(SExp::make_symbol("if"))
            .eval(&mut ctx)
            .unwrap(),
        sym_1()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(sym_2())
            .cons(sym_1())
            .cons(SExp::from(false))
            .cons(SExp::make_symbol("if"))
            .eval(&mut ctx)
            .unwrap(),
        sym_2()
    );
}

#[test]
fn and() {
    let and = || SExp::make_symbol("and");
    let t = || SExp::from(true);
    let f = || SExp::from(false);

    let mut ctx = Context::default();
    assert_eq!(Null.cons(and()).eval(&mut ctx).unwrap(), t());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(t()).cons(t()).cons(and()).eval(&mut ctx).unwrap(),
        t()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(t()).cons(f()).cons(and()).eval(&mut ctx).unwrap(),
        f()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(f()).cons(f()).cons(and()).eval(&mut ctx).unwrap(),
        f()
    );

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((3,))
            .cons(t())
            .cons(and())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(3.0)
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons(Null).cons(SExp::make_symbol("quote")))
            .cons(and())
            .eval(&mut ctx)
            .unwrap(),
        Null
    );

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((and(), ('a', ('b', (false, ('c',))))))
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(false)
    );
}

#[test]
fn or() {
    let or = || SExp::make_symbol("or");
    let t = || SExp::from(true);
    let f = || SExp::from(false);

    let mut ctx = Context::default();
    assert_eq!(Null.cons(or()).eval(&mut ctx).unwrap(), f());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(t()).cons(t()).cons(or()).eval(&mut ctx).unwrap(),
        t()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(t()).cons(f()).cons(or()).eval(&mut ctx).unwrap(),
        t()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(f()).cons(f()).cons(or()).eval(&mut ctx).unwrap(),
        f()
    );

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((or(), (3, (t(),)))).eval(&mut ctx).unwrap(),
        SExp::from(3)
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons(Null).cons(SExp::make_symbol("quote")))
            .cons(or())
            .eval(&mut ctx)
            .unwrap(),
        Null
    );

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((or(), (false, ('a', ('b', ('c',))))))
            .eval(&mut ctx)
            .unwrap(),
        SExp::from('a')
    );
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
