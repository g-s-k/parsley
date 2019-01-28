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
