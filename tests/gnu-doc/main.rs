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

        assert_eq!(s!(ctx, "((lambda (x) (+ x x)) 4)"), SExp::from(8));

        f!(ctx, "lambda.ss");
        assert_eq!(s!(ctx, "(reverse-subtract 7 10)"), SExp::from(3));
        assert_eq!(s!(ctx, "(foo 6)"), SExp::from(10));
    }
}

def_test! {
    named_lambda ctx {
        s!(ctx, "(named-lambda (f x) + x x)");
        assert_eq!(s!(ctx, "((named-lambda (f x) (+ x x)) 4)"), SExp::from(8));
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
        assert_eq!(f!(ctx, "do_2.ss"), SExp::from(25));
    }
}
