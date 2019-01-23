use parsley::prelude::*;
use parsley::Error;


#[test]
fn sicp_2_1() -> Result<(), Error> {
    let mut ctx = Context::base();
    let ex_1 = include_str!("sicp/ch2/ex_1.ss");
    ctx.run(ex_1)?;

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
    let ex_2 = include_str!("sicp/ch2/ex_2.ss");
    ctx.run(ex_2)?;

    assert_eq!(
        ctx.run("(midpoint-segment (make-segment (make-point -5 4) (make-point 7 -2)))")?,
        ctx.run("(make-point 1 1)")?
    );

    Ok(())
}

#[test]
fn sicp_2_3() -> Result<(), Error> {
    let mut ctx = Context::base();
    let ex_3 = include_str!("sicp/ch2/ex_3.ss");
    ctx.run(ex_3)?;

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
