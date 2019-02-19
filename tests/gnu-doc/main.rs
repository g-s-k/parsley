use parsley::{Context, Error, SExp};
use pretty_assertions::assert_eq;

macro_rules! def_test {
    ($name:ident $( $assrt:tt )*) => {
        #[test]
        fn $name() -> Result<(), Error> {
            let mut ctx = Context::base();
            $(
                do_test_step!(ctx, $assrt);
            )*
            Ok(())
        }
    };
}

macro_rules! do_test_step {
    ($ctx:ident, [FILE $file:expr]) => {
        f!($ctx, $file)
    };
    ($ctx:ident, [FILE_EXPR $file:expr, $val:expr]) => {
        assert_eq!(f!($ctx, $file), p!($val))
    };
    ($ctx:ident, [FILE $file:expr, $val:expr]) => {
        assert_eq!(f!($ctx, $file), SExp::from($val))
    };
    ($ctx:ident, [IS_ERR $str:expr]) => {
        assert!($ctx.run($str).is_err())
    };
    ($ctx:ident, [EXPR $str:expr, $val:expr]) => {
        assert_eq!(s!($ctx, $str), p!($val))
    };
    ($ctx:ident, [$str:expr, $val:expr]) => {
        assert_eq!(s!($ctx, $str), SExp::from($val))
    };
    ($ctx:ident, $str:expr) => {
        s!($ctx, $str)
    };
}

macro_rules! f {
    ($ctx:ident, $file:expr) => {
        $ctx.run(include_str!($file))?
    };
}

macro_rules! p {
    ($str:expr) => {
        $str.parse::<SExp>()?
    };
}

macro_rules! s {
    ($ctx:ident, $exp:expr) => {
        $ctx.run($exp)?
    };
}

def_test! {
    lambda
        "(lambda (x) (+ x x))"
        ["((lambda (x) (+ x x)) 4)", 8]

        [FILE "lambda.ss"]
        ["(reverse-subtract 7 10)", 3]
        ["(foo 6)", 10]
}

def_test! {
    named_lambda
        "(named-lambda (f x) (+ x x))"
        ["((named-lambda (f x) (+ x x)) 4)", 8]
}

def_test! {
    r#let
        ["(let ((x 2) (y 3)) (* x y))", 6]
        [FILE "let.ss", 9]
}

def_test! {
    let_star
        [FILE "let*.ss", 70]
}

def_test! {
    letrec
        [FILE "letrec.ss", true]
}

def_test! {
    define_toplevel
        "(define add3 (lambda (x) (+ x 3)))"
        ["(add3 3)", 6]

        "(define first car)"
        ["(first '(1 2))", 1]

        "(define bar)"
        [IS_ERR "bar"]
}

def_test! {
    define_internal
        [FILE "define.ss", 45]
        [FILE "define_letrec.ss", 45]
}

def_test! {
    set
        "(define x 2)"
        ["(+ x 1)", 3]
        "(set! x 4)"
        ["(+ x 1)", 5]
}

def_test! {
    quote
        [EXPR "(quote a)", "a"]
        [EXPR "(quote #(a b c))", "#(a b c)"]
        [EXPR "(quote (+ 1 2))", "(+ 1 2)"]

        [EXPR "'a", "a"]
        [EXPR "'#(a b c)", "#(a b c)"]
        [EXPR "'(+ 1 2)", "(+ 1 2)"]
        [EXPR "'(quote a)", "(quote a)"]
        [EXPR "''a", "(quote a)"]

        [EXPR r#" '"abc" "#, r#" "abc" "#]
        [EXPR r#" "abc" "#, r#" "abc" "#]
        ["'145932", 145932]
        ["145932", 145932]
        ["'#t", true]
        ["#t", true]
        ["'#\\a", 'a']
        ["#\\a", 'a']
}

def_test! {
    quasiquote
        [EXPR "`(list ,(+ 1 2) 4)", "(list 3 4)"]
    // FIXME: quote before quasiquote
        //[EXPR "(let ((name 'a)) `(list ,name ',name))", "(list a 'a)"]
    // FIXME: unquote-splicing
        // [EXPR
        //  "`(a ,(+ 1 2) ,@(map abs '(4 -5 6)) b)",
        //  "(a 3 4 5 b)"
        // ]
    // FIXME: unquote-splicing
        // [EXPR
        //  "`((foo ,(- 10 3)) ,@(cdr '(c)) . ,(car '(cons)))",
        //  "((foo 7) . cons)"
        // ]
    // FIXME: unquote-splicing
        // [EXPR "`#(10 5 ,(sqrt 4) ,@(map sqrt '(16 9)) 8)", "#(10 5 2 4 3 8)"]
    // FIXME: quasiquote with immediate unquote
        // ["`,(+ 2 3)", 5]

    // FIXME: nested quasiquote/unquote
        // [EXPR
        //  "`(a `(b ,(+ 1 2) ,(foo ,(+ 1 3) d) e) f)",
        //  "(a `(b ,(+ 1 2) ,(foo 4 d) e) f)"
        // ]
    // FIXME: nested quasiquote/unquote
        // [EXPR
        //  "(let ((name1 'x) (name2 'y)) `(a `(b ,,name1 ,',name2 d) e))",
        //  "(a `(b ,x ,'y d) e)"
        // ]

        [EXPR "(quasiquote (list (unquote (+ 1 2)) 4))", "(list 3 4)"]
        [EXPR "'(quasiquote (list (unquote (+ 1 2)) 4))", "`(list ,(+ 1 2) 4)"]
}

def_test! {
    r#if
        [EXPR "(if (> 3 2) 'yes 'no)", "yes"]
        [EXPR "(if (> 2 3) 'yes 'no)", "no"]
        [FILE "if.ss", 1]
}

def_test! {
    cond
        [FILE_EXPR "cond_1.ss", "greater"]
        [FILE_EXPR "cond_2.ss", "equal"]
    // FIXME: arrow syntax for cond AND alists
        // ["(cond ((assv 'b '((a 1) (b 2))) => cadr) (else #f))", 2]
}

def_test! {
    case
        [FILE_EXPR "case_1.ss", "composite"]
        [FILE "case_2.ss"]
        [FILE_EXPR "case_3.ss", "consonant"]
}

def_test! {
    and
        ["(and (= 2 2) (> 2 1))", true]
        ["(and (= 2 2) (< 2 1))", false]
        [EXPR "(and 1 2 'c '(f g))", "(f g)"]
        ["(and)", true]
}

def_test! {
    or
        ["(or (= 2 2) (> 2 1))", true]
        ["(or (= 2 2) (< 2 1))", true]
        ["(or #f #f #f)", false]
    // FIXME: implement memq
        // [EXPR "(or (memq 'b '(a b c)) (/ 3 0))", "(b c)"]
}

def_test! {
    begin
        [FILE "begin_1.ss", 6]
        [FILE "begin_2.ss"]
}

def_test! {
    named_let
        [FILE_EXPR "named-let.ss", "((6 1 3) (-5 -2))"]
}

def_test! {
    r#do
        [FILE_EXPR "do_1.ss", "#(0 1 2 3 4)"]
        [FILE "do_2.ss", 25]
}

// TODO: structs and macros

def_test! {
    eqv
        ["(eqv? 'a 'a)", true]
        ["(eqv? 'a 'b)", false]
        ["(eqv? '() '())", true]
        ["(eqv? 100000000 100000000)", true]
        ["(eqv? (cons 1 2) (cons 1 2))", false]
    // FIXME: lambdas with no parameters
        // ["(eqv? (lambda () 1) (lambda () 2))", false]
        ["(eqv? #f 'nil)", false]
    // FIXME: pointer comparisons for procedures
        // ["(let ((p (lambda (x) x))) (eqv? p p))", true]

        r#" (eqv? "" "") "#
        "(eqv? '#() '#())"
        "(eqv? (lambda (x) x) (lambda (x) x))"
        "(eqv? (lambda (x) x) (lambda (y) y))"

        // [FILE "eqv.ss"]
        // ["(let ((g (gen-counter))) (eqv? g g))", true]
        // ["(eqv? (gen-counter) (gen-counter))", false]
        // ["(let ((g (gen-loser))) (eqv? g g))", true]
        // "(eqv? (gen-loser) (gen-loser))"

    // FIXME: lambdas with no parameters
        // "(letrec ((f (lambda () (if (eqv? f g) 'both 'f)))
        //           (g (lambda () (if (eqv? f g) 'both 'g)))
        //    (eqv? f g))"
    // FIXME: lambdas with no parameters
        // ["(letrec ((f (lambda () (if (eqv? f g) 'f 'both)))
        //           (g (lambda () (if (eqv? f g) 'g 'both)))
        //    (eqv? f g))", false]

    // FIXME: pointer comparisons for compound types
        // ["(let ((x '(a))) (eqv? x x))", true]
        "(eqv? '(a) '(a))"
        r#" (eqv? "a" "a") "#
        "(eqv? '(b) (cdr '(a b)))"
}

def_test! {
    eq
        ["(eq? 'a 'a)", true]
        "(eq? '(a) '(a))"
    // FIXME: pointer comparisons for lists
        // ["(eq? (list 'a) (list 'a))", false]
        r#" (eq? "a" "a") "#
        r#" (eq? "" "") "#
        ["(eq? '() '())", true]
        "(eq? 2 2)"
        "(eq? #\\A #\\A)"
    // FIXME: pointer comparisons for procedures
        // ["(eq? car car)", true]
        "(let ((n (+ 2 3))) (eq? n n))"
        ["(let ((x '(a))) (eq? x x))", true]
        ["(let ((x '#())) (eq? x x))", true]
    // FIXME: pointer comparisons for procedures
        // ["(let ((p (lambda (x) x))) (eq? p p))", true]
}

def_test! {
    equal
        ["(equal? 'a 'a)", true]
        ["(equal? '(a) '(a))", true]
        ["(equal? '(a (b) c) '(a (b) c))", true]
        [r#" (equal? "abc" "abc") "#, true]
        ["(equal? 2 2)", true]
    // FIXME: fill parameter for make-vector
        // ["(equal? (make-vector 5 'a) (make-vector 5 'a))", true]
        "(equal? (lambda (x) x) (lambda (y) y))"
}
