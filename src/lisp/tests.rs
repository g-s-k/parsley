#![cfg(test)]

use super::as_atom::AsAtom;
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

    do_parse_and_assert(
        "'potato",
        List(vec![
            SExp::make_symbol("quote"),
            SExp::make_symbol("potato"),
        ]),
    );
}

#[test]
fn eval_empty_list() {
    let mut ctx = Context::default();
    assert!(NULL.eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![SExp::make_symbol("quote"), NULL])
            .eval(&mut ctx)
            .unwrap(),
        NULL
    );
}

#[test]
fn eval_atom() {
    let sym = || SExp::make_symbol("test");
    let quote = || SExp::make_symbol("quote");

    let mut ctx = Context::default();
    assert!(sym().eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert_eq!(List(vec![quote(), sym()]).eval(&mut ctx).unwrap(), sym())
}

#[test]
fn eval_list_quote() {
    let test_list = vec![SExp::make_symbol("quote"), NULL];
    let mut ctx = Context::default();
    assert_eq!(
        List(test_list.clone()).eval(&mut ctx).unwrap(),
        test_list[1].clone()
    );

    let test_list_2 = vec![
        SExp::make_symbol("quote"),
        List(vec![SExp::make_symbol("abc"), SExp::make_symbol("xyz")]),
    ];
    let mut ctx = Context::default();
    assert_eq!(
        List(test_list_2.clone()).eval(&mut ctx).unwrap(),
        test_list_2[1].clone()
    );
}

#[test]
fn eval_null_test() {
    let null = || SExp::make_symbol("null?");
    let quote = || SExp::make_symbol("quote");

    let mut ctx = Context::base();
    assert_eq!(
        List(vec![null(), List(vec![quote(), SExp::make_symbol("test")])])
            .eval(&mut ctx)
            .unwrap(),
        false.as_atom()
    );

    let mut ctx = Context::base();
    assert_eq!(
        List(vec![null(), List(vec![quote(), NULL])])
            .eval(&mut ctx)
            .unwrap(),
        true.as_atom()
    );

    let mut ctx = Context::base();
    assert_eq!(
        List(vec![
            null(),
            List(vec![quote(), List(vec![false.as_atom(), NULL])])
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

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
    assert_eq!(List(vec![and()]).eval(&mut ctx).unwrap(), true.as_atom());

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![and(), true.as_atom(), true.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        true.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![and(), false.as_atom(), true.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        false.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![and(), false.as_atom(), false.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        false.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![and(), true.as_atom(), 3.0.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        3.0.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![and(), List(vec![SExp::make_symbol("quote"), NULL])])
            .eval(&mut ctx)
            .unwrap(),
        NULL
    );

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
    assert_eq!(List(vec![or()]).eval(&mut ctx).unwrap(), false.as_atom());

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![or(), true.as_atom(), true.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        true.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![or(), false.as_atom(), true.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        true.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![or(), false.as_atom(), false.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        false.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![or(), 3.0.as_atom(), true.as_atom()])
            .eval(&mut ctx)
            .unwrap(),
        3.0.as_atom()
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![or(), List(vec![SExp::make_symbol("quote"), NULL])])
            .eval(&mut ctx)
            .unwrap(),
        NULL
    );

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![cond()]).eval(&mut ctx).unwrap(),
        Atom(Primitive::Void)
    );

    let mut ctx = Context::default();
    assert_eq!(
        List(vec![cond(), List(vec![else_(), 'a'.as_atom()])])
            .eval(&mut ctx)
            .unwrap(),
        'a'.as_atom()
    );

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
    assert!(List(vec![begin()]).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
    assert!(List(vec![let_()]).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
    assert!(List(vec![let_(), List(vec![])]).eval(&mut ctx).is_err());

    let mut ctx = Context::default();
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

    let mut ctx = Context::default();
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
