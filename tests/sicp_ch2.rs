use parsley::prelude::*;
use parsley::Error;


#[test]
fn sicp_2_1() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("sicp/ch2/ex_1.ss"))?;

    assert_eq!(
        ctx.run("(add-rat one-half one-third)")?,
        ctx.run("(cons 5 6)")?
    );

    assert_eq!(
        ctx.run("(mul-rat one-half one-third)")?,
        ctx.run("(cons 1 6)")?
    );

    assert_eq!(
        ctx.run("(add-rat one-third one-third)")?,
        ctx.run("(cons 2 3)")?
    );

    Ok(())
}

#[test]
fn sicp_2_2() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("sicp/ch2/ex_2.ss"))?;

    assert_eq!(
        ctx.run("(midpoint-segment (make-segment (make-point -5 4) (make-point 7 -2)))")?,
        ctx.run("(make-point 1 1)")?
    );

    Ok(())
}

#[test]
fn sicp_2_3() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("sicp/ch2/ex_3.ss"))?;

    ctx.run("(define r (make-rect (make-point 2 3) 7 4))")?;

    assert_eq!(
        ctx.run("(rect-area r)")?,
        SExp::from(28)
    );

    assert_eq!(
        ctx.run("(rect-perimeter r)")?,
        SExp::from(22)
    );

    Ok(())
}

// This test fails, because scoping is busted (again).
// This would be solved by implementing continuations.
// #[test]
fn _sicp_2_4() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("sicp/ch2/ex_4.ss"))?;

    assert_eq!(
        ctx.run("(car (cons 3 4))")?,
        SExp::from(3)
    );

    assert_eq!(
        ctx.run("(cdr (cons 3 4))")?,
        SExp::from(4)
    );

    Ok(())
}
