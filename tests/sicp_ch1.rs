use parsley::prelude::*;
use parsley::Error;

// Exercise 1.1
#[test]
fn sicp_1_1() {
    let code = vec![
        "10",
        "(+ 5 3 4)",
        "(- 9 1)",
        "(/ 6 2)",
        "(+ (* 2 4) (- 4 6))",
        "(define a 3)",
        "(define b (+ a 1))",
        "(+ a b (* a b))",
        "(= a b)",
        "(if (and (> b a) (< b (* a b))) b a)",
        "(cond ((= a 4) 6) ((= b 4) (+ 6 7 a)) (else 25))",
        "(+ 2 (if (> b a) b a))",
        "(* (cond ((> a b) a) ((< a b) b) (else -1)) (+ a 1))",
    ];

    let answers = vec![
        "10", "12", "8", "3", "6", "", "", "19", "#f", "4", "16", "6", "16",
    ];

    let mut ctx = Context::base();

    for (c, a) in code.iter().zip(answers.iter()) {
        let parsed = c.parse::<SExp>().unwrap();
        let evaluated = ctx.eval(parsed).unwrap();
        let as_str = format!("{}", evaluated);
        assert_eq!(&as_str, a);
    }
}

// Exercise 1.2
#[test]
fn sicp_1_2() {
    let code = "(/ (+ 5 4 (- 2 (- 3 (+ 6 (/ 4 5))))) (* 3 (- 6 2) (- 2 7)))";
    let answer = -37. / 150.;

    let mut ctx = Context::base();

    let parsed: SExp = code.parse().unwrap();
    let evaluated = ctx.eval(parsed).unwrap();
    assert_eq!(evaluated, SExp::from(answer));
}

// Exercise 1.3
#[test]
fn sicp_1_3() -> Result<(), Error> {
    let mut ctx = Context::base();
    ctx.run(include_str!("./sicp/ch1/ex_3.ss"))?;

    let example_nums = "2 3 4";
    let answer = 25.;
    let invocation = format!("(big-2-sum-sqrs {})", example_nums);

    let evaluated = ctx.run(&invocation)?;
    assert_eq!(evaluated, SExp::from(answer));

    Ok(())
}

// Exercise 1.4
#[test]
fn sicp_1_4() {
    let func = "(define (a-plus-abs-b a b) ((if (> b 0) + -) a b))";

    let mut ctx = Context::base();
    ctx.run(func).unwrap();

    let invoc_1 = "(a-plus-abs-b 3 5)";
    assert_eq!(ctx.run(invoc_1).unwrap(), SExp::from(8));

    let invoc_2 = "(a-plus-abs-b 3 -72.6)";
    assert_eq!(ctx.run(invoc_2).unwrap(), SExp::from(75.6));
}

// TODO: Exercises 1.5 through 1.9

// Exercise 1.10
#[test]
fn sicp_1_10() -> Result<(), Error> {
    let ack = include_str!("./sicp/ch1/ex_5.ss");

    let mut ctx = Context::base();
    ctx.run(ack)?;

    let invoc_1 = "(A 1 10)";
    assert_eq!(ctx.run(invoc_1)?, SExp::from(1024));
    let invoc_2 = "(A 2 4)";
    assert_eq!(ctx.run(invoc_2)?, SExp::from(65536));
    let invoc_3 = "(A 3 3)";
    assert_eq!(ctx.run(invoc_3)?, SExp::from(65536));

    Ok(())
}

// Exercise 1.11
#[test]
fn sicp_1_11() -> Result<(), Error> {
    let func_rec = include_str!("./sicp/ch1/ex_11_rec.ss");
    let func_itr = include_str!("./sicp/ch1/ex_11_iter.ss");

    let mut ctx = Context::base();
    ctx.run(func_rec)?;
    ctx.run(func_itr)?;

    let invoc_r = "(f-r 12)";
    let invoc_i = "(f-i 12)";
    assert_eq!(ctx.run(invoc_r)?, ctx.run(invoc_i)?);

    Ok(())
}

// TODO: Exercises 1.12 through 1.15
