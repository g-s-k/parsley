#![cfg(test)]

use super::SExp::{self, Null};

fn do_parse_and_assert(test_val: &str, expected_val: SExp) {
    let test_parsed = test_val.parse::<SExp>().unwrap();
    assert_eq!(test_parsed, expected_val);
}

#[test]
fn empty_list() {
    do_parse_and_assert("()", Null);
}

#[test]
fn list_of_lists() {
    do_parse_and_assert("(() () ())", Null.cons(Null).cons(Null).cons(Null));
}

#[test]
fn atom() {
    do_parse_and_assert("hello", SExp::make_symbol("hello"));
}

#[test]
fn list_of_atoms() {
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
fn primitive_types() {
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
fn mixed_type_list() {
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
fn quote_syntax() {
    do_parse_and_assert(
        "'(a b c d)",
        Null.cons(
            Null.cons(SExp::make_symbol("d"))
                .cons(SExp::make_symbol("c"))
                .cons(SExp::make_symbol("b"))
                .cons(SExp::make_symbol("a")),
        )
        .cons(SExp::make_symbol("quote")),
    );

    do_parse_and_assert(
        "'potato",
        Null.cons(SExp::make_symbol("potato"))
            .cons(SExp::make_symbol("quote")),
    );
}
