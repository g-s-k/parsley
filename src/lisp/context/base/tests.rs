#![cfg(test)]

use super::*;

#[test]
fn eq_test() {
    let eq = || SExp::make_symbol("eq?");
    let null = || SExp::make_symbol("null");

    assert_eq!(
        SExp::from(vec![eq(), null(), null()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![eq(), null(), 2.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );

    assert_eq!(
        SExp::from(vec![eq(), "woohoo".into(), "woohoo".into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![
            eq(),
            (1 + 2 + 3).into(),
            (9. - 3.5 + 0.25 * 2.).into()
        ])
        .eval(&mut Context::base())
        .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![eq(), (1, (2,)).into(), (1, (2,)).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![eq(), 0.into(), (1, (2,)).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );
}

#[test]
fn null_test() {
    let null = || SExp::make_symbol("null?");
    let null_c = || SExp::make_symbol("null");
    let quote = || SExp::make_symbol("quote");

    assert_eq!(
        SExp::from(vec![
            null(),
            (vec![quote(), SExp::make_symbol("test")]).into()
        ])
        .eval(&mut Context::base())
        .unwrap(),
        SExp::from(false)
    );

    assert_eq!(
        SExp::from(vec![null(), null_c()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![null(), ((quote(), ((),))).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![null(), (vec![quote(), (false, ((),)).into()]).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );
}

#[test]
fn null_const() {
    assert_eq!(
        SExp::make_symbol("null")
            .eval(&mut Context::base())
            .unwrap(),
        Null
    );
}

#[test]
fn not() {
    let not = || SExp::make_symbol("not");

    assert_eq!(
        SExp::from(vec![not(), false.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(true)
    );

    assert_eq!(
        SExp::from(vec![not(), true.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );

    assert_eq!(
        SExp::from(vec![not(), SExp::make_symbol("null")])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );

    assert_eq!(
        SExp::from(vec![not(), (vec![1, 2, 3, 4]).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(false)
    );
}

#[test]
fn cons() {
    let cons = || SExp::make_symbol("cons");
    let item_1 = || SExp::from(5.0);
    let item_2 = || SExp::from("abc");
    let item_3 = || SExp::make_symbol("null");

    // sanity check
    assert_eq!(
        SExp::from((item_1(),)),
        Pair {
            head: Box::new(item_1()),
            tail: Box::new(Null)
        }
    );

    assert_eq!(
        SExp::from(vec![cons(), item_1(), item_3()])
            .eval(&mut Context::base())
            .unwrap(),
        Null.cons(item_1())
    );

    assert_eq!(
        SExp::from(vec![cons(), item_1(), item_2()])
            .eval(&mut Context::base())
            .unwrap(),
        item_2().cons(item_1())
    );

    assert_eq!(
        SExp::from(vec![cons(), item_1(), vec![item_2()].into()])
            .eval(&mut Context::base())
            .unwrap(),
        Null.cons(item_2()).cons(item_1())
    );
}

#[test]
fn car() {
    let car = || SExp::make_symbol("car");

    assert!(SExp::from(Null.cons(Null).cons(car()))
        .eval(&mut Context::base())
        .is_err());

    assert!(SExp::from(Null.cons("test".into()).cons(car()))
        .eval(&mut Context::base())
        .is_err());

    assert_eq!(
        SExp::from(vec![car(), (3, (5,)).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(3)
    )
}

#[test]
fn cdr() {
    let cdr = || SExp::make_symbol("cdr");

    assert!(SExp::from(Null.cons(Null).cons(cdr()))
        .eval(&mut Context::base())
        .is_err());

    assert!(SExp::from(Null.cons("test".into()).cons(cdr()))
        .eval(&mut Context::base())
        .is_err());

    assert_eq!(
        SExp::from(vec![cdr(), (3, (5,)).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from((5,))
    )
}

#[test]
fn type_of() {
    let tpf = || SExp::make_symbol("type-of");

    assert_eq!(
        SExp::from(vec![tpf(), SExp::make_symbol("null")])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![
            tpf(),
            Null.cons(Null).cons(SExp::make_symbol("quote"))
        ])
        .eval(&mut Context::base())
        .unwrap(),
    );

    // ha, get it
    assert_eq!(
        SExp::from(vec![tpf(), 3.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), std::f64::consts::PI.into()])
            .eval(&mut Context::base())
            .unwrap(),
    );

    assert_eq!(
        SExp::from(vec![tpf(), 'b'.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), '\n'.into()])
            .eval(&mut Context::base())
            .unwrap(),
    );

    assert_eq!(
        SExp::from(vec![tpf(), true.into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), false.into()])
            .eval(&mut Context::base())
            .unwrap(),
    );

    assert_eq!(
        SExp::from(vec![tpf(), "yes".into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), "potato".into()])
            .eval(&mut Context::base())
            .unwrap(),
    );

    assert_eq!(
        SExp::from(vec![tpf(), SExp::make_symbol("null?")])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), SExp::make_symbol("+")])
            .eval(&mut Context::base())
            .unwrap(),
    );

    assert_eq!(
        SExp::from(vec![tpf(), ("abc", (123,)).into()])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), (false, ('\0',)).into()])
            .eval(&mut Context::base())
            .unwrap(),
    );
}
