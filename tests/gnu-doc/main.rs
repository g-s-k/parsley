use parsley::{Context, Error, SExp};

macro_rules! def_test {
    ($name:ident $ctx:ident $body:block) => {
        #[test]
        fn $name() -> Result<(), Error> {
            let mut $ctx = Context::base();
            $body
            Ok(())
        }
    };
}

macro_rules! a {
    ($exp:expr, $val:expr) => { assert_eq!($exp, SExp::from($val)) };
}

macro_rules! ae {
    ($ctx:ident, $str:expr) => { assert!($ctx.run($str).is_err()) };
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
    lambda ctx {
        s!(ctx, "(lambda (x) (+ x x))");

        a!(s!(ctx, "((lambda (x) (+ x x)) 4)"), 8);

        f!(ctx, "lambda.ss");
        a!(s!(ctx, "(reverse-subtract 7 10)"), 3);
        a!(s!(ctx, "(foo 6)"), 10);
    }
}

def_test! {
    named_lambda ctx {
        s!(ctx, "(named-lambda (f x) + x x)");
        a!(s!(ctx, "((named-lambda (f x) (+ x x)) 4)"), 8);
    }
}

def_test! {
    r#let ctx {
        a!(s!(ctx, "(let ((x 2) (y 3)) (* x y))"), 6);
        a!(f!(ctx, "let.ss"), 9);
    }
}

def_test! {
    let_star ctx {
        a!(f!(ctx, "let*.ss"), 70);
    }
}

def_test! {
    letrec ctx {
        a!(f!(ctx, "letrec.ss"), true);
    }
}

def_test! {
    define_top_level ctx {
        s!(ctx, "(define add3 (lambda (x) (+ x 3)))");
        a!(s!(ctx, "(add3 3)"), 6);

        s!(ctx, "(define first car)");
        a!(s!(ctx, "(first '(1 2))"), 1);

        s!(ctx, "(define bar)");
        ae!(ctx, "bar");
    }
}

def_test! {
    define_internal ctx {
        a!(f!(ctx, "define.ss"), 45);
        a!(f!(ctx, "define_letrec.ss"), 45);
    }
}

def_test! {
    named_let ctx {
        assert_eq!(
            f!(ctx, "named-let.ss"),
            p!("((6 1 3) (-5 -2))")
        );
    }
}

def_test! {
    r#do ctx {
        assert_eq!(
            f!(ctx, "do_1.ss"),
            p!("#(0 1 2 3 4)")
        );
        a!(f!(ctx, "do_2.ss"), 25);
    }
}
