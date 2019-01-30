//! This is a demonstration of exactly what we sacrifice for the increased
//! range and precision of the adaptive `Num` type.

#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use parsley::Num;
    use test::{black_box, Bencher};

    #[bench]
    fn add_int_raw(b: &mut Bencher) {
        let three = 3;
        let five = 5;
        b.iter(|| {
            for _ in 0..100 {
                black_box(three + five);
            }
        })
    }

    #[bench]
    fn add_int_num(b: &mut Bencher) {
        let three = Num::from(3);
        let five = Num::from(5);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three + five);
            }
        })
    }

    #[bench]
    fn add_float_raw(b: &mut Bencher) {
        let three = 3.;
        let five = 5.;
        b.iter(|| {
            for _ in 0..100 {
                black_box(three + five);
            }
        })
    }

    #[bench]
    fn add_float_num(b: &mut Bencher) {
        let three = Num::from(3.);
        let five = Num::from(5.);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three + five);
            }
        })
    }

    #[bench]
    fn div_int_raw(b: &mut Bencher) {
        let three = 3;
        let five = 5;
        b.iter(|| {
            for _ in 0..100 {
                black_box(three / five);
            }
        })
    }

    #[bench]
    fn div_int_num(b: &mut Bencher) {
        let three = Num::from(3);
        let five = Num::from(5);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three / five);
            }
        })
    }

    #[bench]
    fn div_float_raw(b: &mut Bencher) {
        let three = 3.;
        let five = 5.;
        b.iter(|| {
            for _ in 0..100 {
                black_box(three / five);
            }
        })
    }

    #[bench]
    fn div_float_num(b: &mut Bencher) {
        let three = Num::from(3.);
        let five = Num::from(5.);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three / five);
            }
        })
    }

    #[bench]
    fn div_int_raw_no_rem(b: &mut Bencher) {
        let fifteen = 15;
        let five = 5;
        b.iter(|| {
            for _ in 0..100 {
                black_box(fifteen / five);
            }
        })
    }

    #[bench]
    fn div_int_num_no_rem(b: &mut Bencher) {
        let fifteen = Num::from(15);
        let five = Num::from(5);
        b.iter(|| {
            for _ in 0..100 {
                black_box(fifteen / five);
            }
        })
    }

    #[bench]
    fn hypot_raw(b: &mut Bencher) {
        let three = 3.;
        let five = 5.;
        b.iter(|| {
            for _ in 0..100 {
                black_box(f64::hypot(three, five));
            }
        })
    }

    #[bench]
    fn hypot_float_num(b: &mut Bencher) {
        let three = Num::from(3.);
        let five = Num::from(5.);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three.hypot(five));
            }
        })
    }

    #[bench]
    fn hypot_int_num(b: &mut Bencher) {
        let three = Num::from(3);
        let five = Num::from(5);
        b.iter(|| {
            for _ in 0..100 {
                black_box(three.hypot(five));
            }
        })
    }
}
