use parsley::{Context, Error};

#[test]
fn fizzbuzz() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("fizzbuzz.ss"))?;
    Ok(())
}
