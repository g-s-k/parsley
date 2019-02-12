use parsley::{Context, Error, SExp};

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
    named_let
        [FILE_EXPR "named-let.ss", "((6 1 3) (-5 -2))"]
}

def_test! {
    r#do
        [FILE_EXPR "do_1.ss", "#(0 1 2 3 4)"]
        [FILE "do_2.ss", 25]
}
