#![cfg(test)]

use super::*;

fn eval(e: SExp) -> Result {
    Context::base().eval(e)
}

#[test]
fn eq_test() {
    let eq = || SExp::sym("eq?");
    let null = || SExp::sym("null");

    assert_eq!(eval(sexp![eq(), null(), null()]).unwrap(), SExp::from(true));

    assert_eq!(eval(sexp![eq(), null(), 2]).unwrap(), SExp::from(false));

    assert_eq!(
        eval(sexp![eq(), "woohoo", "woohoo"]).unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        eval(sexp![eq(), 1 + 2 + 3, 9. - 3.5 + 0.25 * 2.]).unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        eval(sexp![eq(), (1, (2,)), (1, (2,))]).unwrap(),
        SExp::from(true)
    );

    assert_eq!(eval(sexp![eq(), 0, (1, (2,))]).unwrap(), SExp::from(false));
}

#[test]
fn null_test() {
    let null = || SExp::sym("null?");
    let null_c = || SExp::sym("null");
    let quote = || SExp::sym("quote");

    assert_eq!(
        eval(sexp![null(), sexp![quote(), SExp::sym("test")]]).unwrap(),
        SExp::from(false)
    );

    assert_eq!(eval(sexp![null(), null_c()]).unwrap(), SExp::from(true));

    assert_eq!(
        eval(sexp![null(), (quote(), ((),))]).unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        eval(sexp![null(), sexp![quote(), (false, ((),))]]).unwrap(),
        SExp::from(false)
    );
}

#[test]
fn null_const() {
    assert_eq!(eval(SExp::sym("null")).unwrap(), Null);
}

#[test]
fn not() {
    let not = || SExp::sym("not");

    assert_eq!(eval(sexp![not(), false]).unwrap(), SExp::from(true));

    assert_eq!(eval(sexp![not(), true]).unwrap(), SExp::from(false));

    assert_eq!(
        eval(sexp![not(), SExp::sym("null")]).unwrap(),
        SExp::from(false)
    );

    assert_eq!(
        eval(sexp![not(), vec![1, 2, 3, 4]]).unwrap(),
        SExp::from(false)
    );
}

#[test]
fn cons() {
    let cons = || SExp::sym("cons");
    let item_1 = || SExp::from(5.0);
    let item_2 = || SExp::from("abc");
    let item_3 = || SExp::sym("null");

    // sanity check
    assert_eq!(
        SExp::from((item_1(),)),
        Pair {
            head: Box::new(item_1()),
            tail: Box::new(Null)
        }
    );

    assert_eq!(
        eval(sexp![cons(), item_1(), item_3()]).unwrap(),
        Null.cons(item_1())
    );

    assert_eq!(
        eval(sexp![cons(), item_1(), item_2()]).unwrap(),
        item_2().cons(item_1())
    );

    assert_eq!(
        eval(sexp![cons(), item_1(), vec![item_2()]]).unwrap(),
        Null.cons(item_2()).cons(item_1())
    );
}

#[test]
fn car() {
    let car = || SExp::sym("car");

    assert!(eval(SExp::from(Null.cons(Null).cons(car()))).is_err());

    assert!(eval(SExp::from(Null.cons("test".into()).cons(car()))).is_err());

    assert_eq!(eval(sexp![car(), (3, (5,))]).unwrap(), SExp::from(3))
}

#[test]
fn cdr() {
    let cdr = || SExp::sym("cdr");

    assert!(eval(SExp::from(Null.cons(Null).cons(cdr()))).is_err());

    assert!(eval(SExp::from(Null.cons("test".into()).cons(cdr()))).is_err());

    assert_eq!(eval(sexp![cdr(), (3, (5,))]).unwrap(), SExp::from((5,)))
}

#[test]
fn type_of() {
    let tpf = || SExp::sym("type-of");

    assert_eq!(
        eval(sexp![tpf(), SExp::sym("null")]).unwrap(),
        eval(sexp![tpf(), Null.cons(Null).cons(SExp::sym("quote"))]).unwrap(),
    );

    // ha, get it
    assert_eq!(
        eval(sexp![tpf(), 3]).unwrap(),
        eval(sexp![tpf(), std::f64::consts::PI]).unwrap(),
    );

    assert_eq!(
        eval(sexp![tpf(), 'b']).unwrap(),
        eval(sexp![tpf(), '\n']).unwrap(),
    );

    assert_eq!(
        eval(sexp![tpf(), true]).unwrap(),
        eval(sexp![tpf(), false]).unwrap(),
    );

    assert_eq!(
        eval(sexp![tpf(), "yes"]).unwrap(),
        eval(sexp![tpf(), "potato"]).unwrap(),
    );

    assert_eq!(
        eval(sexp![tpf(), SExp::sym("null?")]).unwrap(),
        eval(sexp![tpf(), SExp::sym("+")]).unwrap(),
    );

    assert_eq!(
        eval(sexp![tpf(), ("abc", (123,))]).unwrap(),
        eval(sexp![tpf(), (false, ('\0',))]).unwrap(),
    );
}
