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
    do_parse_and_assert("hello", SExp::sym("hello"));
}

#[test]
fn list_of_atoms() {
    do_parse_and_assert(
        "(a bc de fgh ijk l mnop)",
        Null.cons(SExp::sym("mnop"))
            .cons(SExp::sym("l"))
            .cons(SExp::sym("ijk"))
            .cons(SExp::sym("fgh"))
            .cons(SExp::sym("de"))
            .cons(SExp::sym("bc"))
            .cons(SExp::sym("a")),
    );
}

#[test]
fn comments() {
    do_parse_and_assert(
        r#"
; leading comment
(1 ;; double semicolon
(2 null)
; in between
(x)
;; not included: 5)
)
"#,
        Null.cons(Null.cons(SExp::sym("x")))
            .cons(Null.cons(SExp::sym("null")).cons(2.into()))
            .cons(1.into()),
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
            Null.cons(SExp::sym("d"))
                .cons(SExp::sym("c"))
                .cons(SExp::sym("b"))
                .cons(SExp::sym("a")),
        )
        .cons(SExp::sym("quote")),
    );

    do_parse_and_assert(
        "'potato",
        Null.cons(SExp::sym("potato")).cons(SExp::sym("quote")),
    );
}
