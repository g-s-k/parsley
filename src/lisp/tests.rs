#![cfg(test)]

use super::SExp::{self, Atom, Null, Pair};
use super::*;

fn do_parse_and_assert(test_val: &str, expected_val: SExp) {
    let test_parsed = test_val.parse::<SExp>().unwrap();
    assert_eq!(test_parsed, expected_val);
}

#[test]
fn parse_empty_list() {
    do_parse_and_assert("()", Null);
}

#[test]
fn parse_list_of_lists() {
    do_parse_and_assert("(() () ())", Null.cons(Null).cons(Null).cons(Null));
}

#[test]
fn parse_atom() {
    do_parse_and_assert("hello", SExp::make_symbol("hello"));
}

#[test]
fn parse_list_of_atoms() {
    do_parse_and_assert(
        "(a bc de fgh ijk l mnop)",
        Null.cons(SExp::make_symbol("mnop"))
            .cons(SExp::make_symbol("l"))
            .cons(SExp::make_symbol("ijk"))
            .cons(SExp::make_symbol("fgh"))
            .cons(SExp::make_symbol("de"))
            .cons(SExp::make_symbol("bc"))
            .cons(SExp::make_symbol("a")),
    );
}

#[test]
fn parse_primitive_types() {
    do_parse_and_assert("#f", SExp::from(false));
    do_parse_and_assert("#t", SExp::from(true));
    do_parse_and_assert("0", SExp::from(0));
    do_parse_and_assert("2.0", SExp::from(2));
    do_parse_and_assert("inf", SExp::from(std::f64::INFINITY));
    do_parse_and_assert("-inf", SExp::from(std::f64::NEG_INFINITY));
    do_parse_and_assert("#\\c", SExp::from('c'));
    do_parse_and_assert("#\\'", SExp::from('\''));
    do_parse_and_assert(
        r#""test string with spaces""#,
        SExp::from("test string with spaces"),
    );
}

#[test]
fn parse_mixed_type_list() {
    do_parse_and_assert(
        "(0 #f () 33.5 \"xyz\" #\\? #t \"\" \"   \")",
        Null.cons(SExp::from("   "))
            .cons(SExp::from(""))
            .cons(SExp::from(true))
            .cons(SExp::from('?'))
            .cons(SExp::from("xyz"))
            .cons(SExp::from(33.5))
            .cons(Null)
            .cons(SExp::from(false))
            .cons(SExp::from(0)),
    );
}

#[test]
fn parse_quote_syntax() {
    do_parse_and_assert(
        "'(a b c d)",
        Null.cons(SExp::make_symbol("d"))
            .cons(SExp::make_symbol("c"))
            .cons(SExp::make_symbol("b"))
            .cons(SExp::make_symbol("a"))
            .cons(SExp::make_symbol("quote")),
    );

    do_parse_and_assert(
        "'potato",
        Null.cons(SExp::make_symbol("potato"))
            .cons(SExp::make_symbol("quote")),
    );
}

#[test]
fn eval_empty_list() {
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
fn eval_atom() {
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
fn eval_list_quote() {
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
fn eval_null_test() {
    let null = || SExp::make_symbol("null?");
    let quote = || SExp::make_symbol("quote");

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(Null.cons(SExp::make_symbol("test")).cons(quote()))
            .cons(null())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(false)
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(Null.cons(Null).cons(quote()))
            .cons(null())
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(true)
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(
            Null.cons(Null.cons(Null).cons(SExp::from(false)))
                .cons(quote())
        )
        .cons(null())
        .eval(&mut ctx)
        .unwrap(),
        SExp::from(false)
    );
}

#[test]
fn eval_cons() {
    let cons = || SExp::make_symbol("cons");
    let item_1 = || SExp::from(5.0);
    let item_2 = || SExp::from("abc");
    let item_3 = || SExp::make_symbol("null");

    // sanity check
    assert_eq!(
        Null.cons(item_1()),
        Pair {
            head: Box::new(item_1()),
            tail: Box::new(Null)
        }
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(item_3())
            .cons(item_1())
            .cons(cons())
            .eval(&mut ctx)
            .unwrap(),
        Null.cons(item_1())
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(item_2())
            .cons(item_1())
            .cons(cons())
            .eval(&mut ctx)
            .unwrap(),
        item_2().cons(item_1())
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(Null.cons(item_2()))
            .cons(item_1())
            .cons(cons())
            .eval(&mut ctx)
            .unwrap(),
        Null.cons(item_2()).cons(item_1())
    );
}

#[test]
fn eval_if() {
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
fn eval_and() {
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
fn eval_or() {
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
        SExp::from((or(), (3, (t(),))))
            .eval(&mut ctx)
            .unwrap(),
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
fn eval_cond() {
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
fn eval_begin() {
    let begin = || SExp::make_symbol("begin");

    let mut ctx = Context::default();
    assert!(Null.cons(begin()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        SExp::from((begin(), (0, (1,))))
            .eval(&mut ctx)
            .unwrap(),
        SExp::from(1)
    )
}

#[test]
fn eval_let() {
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
