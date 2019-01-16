//! This serves as some reasoning for the way the `sexp` macro is implemented -
//! it seems like using a slice as an intermediate container for values to be
//! iterated over is the most efficient way to do it.

#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use test::{black_box, Bencher};

    macro_rules! with_vec_push {
        ( $( $e:expr ),* ) => {{
            let mut tmp = Vec::new();
            $(
                tmp.push($e);
            )*
            tmp.into_iter().collect::<Vec<_>>()
        }};
    }

    #[bench]
    fn vec_push(b: &mut Bencher) {
        b.iter(|| {
            black_box(with_vec_push![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
            ]);
        })
    }

    macro_rules! with_vec {
        ( $( $e:expr ),* ) => { (vec![ $( $e ),* ]).into_iter().collect::<Vec<_>>() };
    }

    #[bench]
    fn vec(b: &mut Bencher) {
        b.iter(|| {
            black_box(with_vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
            ]);
        })
    }

    macro_rules! with_slice {
        ( $( $e:expr ),* ) => { (&[ $( $e ),* ][..]).into_iter().collect::<Vec<_>>() };
    }

    #[bench]
    fn slice(b: &mut Bencher) {
        b.iter(|| {
            black_box(with_slice![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
            ]);
        })
    }
}
