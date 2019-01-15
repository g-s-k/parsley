use parsley::prelude::*;

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
        let evaluated = parsed.eval(&mut ctx).unwrap();
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
    let evaluated = parsed.eval(&mut ctx).unwrap();
    assert_eq!(evaluated, SExp::from(answer));
}

// Exercise 1.3
#[test]
fn sicp_1_3() {
    let func = r#"
(define (sqr x) (* x x))
(define (sumsqr x y) (+ (sqr x) (sqr y)))

(define (biggest-2-of-3 x y z)
 (if
  (> y x)
  (if (> z x) (list z y) (list x y))
  (if (> z y) (list z x) (list x y))
  )
 )

(define (big-2-sum-sqrs x y z)
 (define big-2 (biggest-2-of-3 x y z))
 (sumsqr (car big-2) (car (cdr big-2)))
 )
"#;

    let mut ctx = Context::base();
    func.parse::<SExp>().unwrap().eval(&mut ctx).unwrap();

    let example_nums = "2 3 4";
    let answer = 25.;
    let invocation = format!("(big-2-sum-sqrs {})", example_nums);

    let parsed: SExp = invocation.parse().unwrap();
    let evaluated = parsed.eval(&mut ctx).unwrap();
    assert_eq!(evaluated, SExp::from(answer));
}

// Exercise 1.4
#[test]
fn sicp_1_4() {
    let func = "(define (a-plus-abs-b a b) ((if (> b 0) + -) a b))";

    let mut ctx = Context::base();
    func.parse::<SExp>().unwrap().eval(&mut ctx).unwrap();

    let invoc_1 = "(a-plus-abs-b 3 5)";
    assert_eq!(
        invoc_1.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        SExp::from(8)
    );

    let invoc_2 = "(a-plus-abs-b 3 -72.6)";
    assert_eq!(
        invoc_2.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        SExp::from(75.6)
    );
}

// TODO: Exercises 1.5 through 1.9

// Exercise 1.10
#[test]
fn sicp_1_10() {
    let ack = r#"
(define (A x y)
 (cond
  ((= y 0) 0)
  ((= x 0) (* 2 y))
  ((= y 1) 2)
  (else (A (- x 1) (A x (- y 1))))
  )
)
"#;

    let mut ctx = Context::base();
    ack.parse::<SExp>().unwrap().eval(&mut ctx).unwrap();

    let invoc_1 = "(A 1 10)";
    assert_eq!(
        invoc_1.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        SExp::from(1024)
    );
    let invoc_2 = "(A 2 4)";
    assert_eq!(
        invoc_2.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        SExp::from(65536)
    );
    let invoc_3 = "(A 3 3)";
    assert_eq!(
        invoc_3.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        SExp::from(65536)
    );
}

// Exercise 1.11
#[test]
fn sicp_1_11() {
    let func_rec = r#"
(define (f-r n)
 (if
  (< n 3)
  n
  (+ (f-r (- n 1)) (* 2 (f-r (- n 2))) (* 3 (f-r (- n 3))))
  )
 )
"#;

    let func_itr = r#"
(define (f-i-calc a b c) (+ a (* 2 b) (* 3 c)))

(define (f-i-helper f-3 f-2 f-1 m)
  (if
   (= m n)
   (f-i-calc f-1 f-2 f-3)
   (f-i-helper f-2 f-1 (f-i-calc f-1 f-2 f-3) (+ m 1))
   )
 )

(define (f-i n)
 (f-i-helper 0 1 2 3)
 )
"#;

    let mut ctx = Context::base();
    func_rec.parse::<SExp>().unwrap().eval(&mut ctx).unwrap();
    func_itr.parse::<SExp>().unwrap().eval(&mut ctx).unwrap();

    let invoc_r = "(f-r 12)";
    let invoc_i = "(f-i 12)";
    assert_eq!(
        invoc_r.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
        invoc_i.parse::<SExp>().unwrap().eval(&mut ctx).unwrap(),
    );
}

// TODO: Exercises 1.12 through 1.15
