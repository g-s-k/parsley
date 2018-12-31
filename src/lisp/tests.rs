#![cfg(test)]

use super::as_atom::AsAtom;
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
    do_parse_and_assert("#f", false.as_atom());
    do_parse_and_assert("#t", true.as_atom());
    do_parse_and_assert("0", 0_f64.as_atom());
    do_parse_and_assert("2.0", 2.0.as_atom());
    do_parse_and_assert("inf", std::f64::INFINITY.as_atom());
    do_parse_and_assert("-inf", std::f64::NEG_INFINITY.as_atom());
    do_parse_and_assert("#\\c", 'c'.as_atom());
    do_parse_and_assert("#\\'", '\''.as_atom());
    do_parse_and_assert(
        r#""test string with spaces""#,
        "test string with spaces".as_atom(),
    );
}

#[test]
fn parse_mixed_type_list() {
    do_parse_and_assert(
        "(0 #f () 33.5 \"xyz\" #\\? #t \"\" \"   \")",
        Null.cons("   ".as_atom())
            .cons("".as_atom())
            .cons(true.as_atom())
            .cons('?'.as_atom())
            .cons("xyz".as_atom())
            .cons(33.5.as_atom())
            .cons(Null)
            .cons(false.as_atom())
            .cons(0_f64.as_atom()),
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
        false.as_atom()
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(Null.cons(Null).cons(quote()))
            .cons(null())
            .eval(&mut ctx)
            .unwrap(),
        true.as_atom()
    );

    let mut ctx = Context::base();
    assert_eq!(
        Null.cons(
            Null.cons(Null.cons(Null).cons(false.as_atom()))
                .cons(quote())
        )
        .cons(null())
        .eval(&mut ctx)
        .unwrap(),
        false.as_atom()
    );
}

#[test]
fn eval_cons() {
    let cons = || SExp::make_symbol("cons");
    let item_1 = || 5.0.as_atom();
    let item_2 = || "abc".as_atom();
    let item_3 = || SExp::make_symbol("null");

    // sanity check
    assert_eq!(
        Null.cons(item_1()),
        Pair {
            head: box item_1(),
            tail: box Null
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
    let sym_1 = || "one".as_atom();
    let sym_2 = || "two".as_atom();

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(sym_2())
            .cons(sym_1())
            .cons(true.as_atom())
            .cons(SExp::make_symbol("if"))
            .eval(&mut ctx)
            .unwrap(),
        sym_1()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(sym_2())
            .cons(sym_1())
            .cons(false.as_atom())
            .cons(SExp::make_symbol("if"))
            .eval(&mut ctx)
            .unwrap(),
        sym_2()
    );
}

#[test]
fn eval_and() {
    let and = || SExp::make_symbol("and");
    let t = || true.as_atom();
    let f = || false.as_atom();

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
        Null.cons(3.0.as_atom())
            .cons(t())
            .cons(and())
            .eval(&mut ctx)
            .unwrap(),
        3.0.as_atom()
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
        Null.cons('c'.as_atom())
            .cons(false.as_atom())
            .cons('b'.as_atom())
            .cons('a'.as_atom())
            .cons(and())
            .eval(&mut ctx)
            .unwrap(),
        false.as_atom()
    );
}

#[test]
fn eval_or() {
    let or = || SExp::make_symbol("or");
    let t = || true.as_atom();
    let f = || false.as_atom();

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
        Null.cons(t())
            .cons(3.0.as_atom())
            .cons(or())
            .eval(&mut ctx)
            .unwrap(),
        3.0.as_atom()
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
        Null.cons('c'.as_atom())
            .cons('b'.as_atom())
            .cons('a'.as_atom())
            .cons(false.as_atom())
            .cons(or())
            .eval(&mut ctx)
            .unwrap(),
        'a'.as_atom()
    );
}

#[test]
fn eval_cond() {
    let cond = || SExp::make_symbol("cond");
    let else_ = || SExp::make_symbol("else");
    let t = || true.as_atom();
    let f = || false.as_atom();

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(cond()).eval(&mut ctx).unwrap(),
        Atom(Primitive::Void)
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons('a'.as_atom()).cons(else_()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        'a'.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons('a'.as_atom()).cons(else_()))
            .cons(Null.cons('b'.as_atom()).cons(t()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        'b'.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons('b'.as_atom()).cons(f()))
            .cons(Null.cons('a'.as_atom()).cons(else_()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        'a'.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons('a'.as_atom()).cons(else_()))
            .cons(Null.cons('d'.as_atom()).cons(t()))
            .cons(Null.cons('b'.as_atom()).cons(t()))
            .cons(Null.cons('c'.as_atom()).cons(f()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        'b'.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(Null.cons('a'.as_atom()).cons(else_()))
            .cons(Null.cons('d'.as_atom()).cons(f()))
            .cons(Null.cons('b'.as_atom()).cons(f()))
            .cons(Null.cons('c'.as_atom()).cons(f()))
            .cons(cond())
            .eval(&mut ctx)
            .unwrap(),
        'a'.as_atom()
    );
}

#[test]
fn eval_begin() {
    let begin = || SExp::make_symbol("begin");

    let mut ctx = Context::default();
    assert!(Null.cons(begin()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(1_f64.as_atom())
            .cons(0_f64.as_atom())
            .cons(begin())
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

    let mut ctx = Context::default();
    assert!(Null.cons(let_()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert!(Null.cons(Null).cons(let_()).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(x())
            .cons(Null.cons(Null.cons(3_f64.as_atom()).cons(x())))
            .cons(let_())
            .eval(&mut ctx)
            .unwrap(),
        3_f64.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        Null.cons(y())
            .cons(x())
            .cons(
                Null.cons(Null.cons(5_f64.as_atom()).cons(y()))
                    .cons(Null.cons(3_f64.as_atom()).cons(x()))
            )
            .cons(let_())
            .eval(&mut ctx)
            .unwrap(),
        5_f64.as_atom()
    );
}
