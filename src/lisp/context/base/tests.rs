#![cfg(test)]

use super::*;

#[test]
fn eq_test() {
    let eq = || SExp::sym("eq?");
    let null = || SExp::sym("null");

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
    let null = || SExp::sym("null?");
    let null_c = || SExp::sym("null");
    let quote = || SExp::sym("quote");

    assert_eq!(
        SExp::from(vec![
            null(),
            (vec![quote(), SExp::sym("test")]).into()
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
        SExp::sym("null")
            .eval(&mut Context::base())
            .unwrap(),
        Null
    );
}

#[test]
fn not() {
    let not = || SExp::sym("not");

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
        SExp::from(vec![not(), SExp::sym("null")])
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
    let car = || SExp::sym("car");

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
    let cdr = || SExp::sym("cdr");

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
    let tpf = || SExp::sym("type-of");

    assert_eq!(
        SExp::from(vec![tpf(), SExp::sym("null")])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![
            tpf(),
            Null.cons(Null).cons(SExp::sym("quote"))
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
        SExp::from(vec![tpf(), SExp::sym("null?")])
            .eval(&mut Context::base())
            .unwrap(),
        SExp::from(vec![tpf(), SExp::sym("+")])
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
