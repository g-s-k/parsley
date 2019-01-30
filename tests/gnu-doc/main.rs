use parsley::{Context, Error, SExp};

#[test]
fn named_let() -> Result<(), Error> {
    let mut ctx = Context::base();
    assert_eq!(
        ctx.run(include_str!("named-let.ss"))?,
        "((6 1 3) (-5 -2))".parse::<SExp>()?
    );
    Ok(())
}

#[test]
fn r#do() -> Result<(), Error> {
    let mut ctx = Context::base();
    assert_eq!(
        ctx.run(include_str!("do_1.ss"))?,
        "#(0 1 2 3 4)".parse::<SExp>()?
    );
    assert_eq!(ctx.run(include_str!("do_2.ss"))?, SExp::from(25));
    Ok(())
}
