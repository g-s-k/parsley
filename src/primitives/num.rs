#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use std::f64::{EPSILON, INFINITY, NEG_INFINITY};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

use super::super::Error;
use self::Num::{Float, Int};

type IntT = isize;

/// A numeric type that adapts its precision based on its usage.
#[derive(Clone, Copy, Debug, PartialOrd)]
pub enum Num {
    Float(f64),
    Int(IntT),
}

impl Num {
    pub fn abs(self) -> Self {
        match self {
            Float(f) => Float(f.abs()),
            Int(i) => Int(i.abs()),
        }
    }

    pub fn pow<T>(self, other: T) -> Self
    where
        Self: From<T>,
    {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Int(i0.pow(i1 as u32)),
            (Float(f), Int(i)) => Float(f.powi(i as i32)),
            (Int(i), Float(f)) => Int(i.pow(f as u32)),
            (Float(f0), Float(f1)) => Float(f0.powf(f1)),
        }
    }
}

impl FromStr for Num {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<IntT>() {
            return Ok(Int(num));
        }

        if let Ok(num) = s.parse::<f64>() {
            return Ok(Float(num));
        }

        Err(Error::Syntax { exp: s.to_string() })
    }
}

impl From<IntT> for Num {
    fn from(n: IntT) -> Self {
        Num::Int(n)
    }
}

impl From<i32> for Num {
    fn from(n: i32) -> Self {
        Num::Int(n as IntT)
    }
}

impl From<usize> for Num {
    fn from(n: usize) -> Self {
        Num::Int(n as IntT)
    }
}

impl From<f32> for Num {
    fn from(n: f32) -> Self {
        Num::Float(f64::from(n))
    }
}

impl From<f64> for Num {
    fn from(n: f64) -> Self {
        Num::Float(n)
    }
}

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Int(i0), Int(i1)) => i0 == i1,
            (Float(f), Int(i)) | (Int(i), Float(f)) => (f - (i as f64)).abs() < EPSILON,
            (Float(f0), Float(f1)) if f0 == INFINITY && f1 == INFINITY => true,
            (Float(f0), Float(f1)) if f0 == NEG_INFINITY && f1 == NEG_INFINITY => true,
            (Float(f0), Float(f1)) => (f0 - f1).abs() < EPSILON,
        }
    }
}

impl From<Num> for usize {
    fn from(n: Num) -> Self {
        match n {
            Num::Float(f) => f as Self,
            Num::Int(i) => i as Self,
        }
    }
}

impl From<Num> for f64 {
    fn from(n: Num) -> Self {
        match n {
            Num::Float(f) => f,
            Num::Int(i) => i as Self,
        }
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Float(l) => write!(f, "{}", l),
            Int(i) => write!(f, "{}", i),
        }
    }
}

impl Neg for Num {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Int(i) => Int(-i),
            Float(f) => Float(-f),
        }
    }
}

impl<T> Add<T> for Num
where
    Num: From<T>,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Int(i0 + i1),
            (Float(f), Int(i)) | (Int(i), Float(f)) => Float(f + (i as f64)),
            (Float(f0), Float(f1)) => Float(f0 + f1),
        }
    }
}

impl<T> Sub<T> for Num
where
    Num: From<T>,
{
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Int(i0 - i1),
            (Float(f), Int(i)) => Float(f - (i as f64)),
            (Int(i), Float(f)) => Float((i as f64) - f),
            (Float(f0), Float(f1)) => Float(f0 - f1),
        }
    }
}

impl<T> Mul<T> for Num
where
    Num: From<T>,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Int(i0 * i1),
            (Float(f), Int(i)) | (Int(i), Float(f)) => Float(f * (i as f64)),
            (Float(f0), Float(f1)) => Float(f0 * f1),
        }
    }
}

impl<T> Div<T> for Num
where
    Num: From<T>,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Float(i0 as f64 / i1 as f64),
            (Float(f), Int(i)) => Float(f / (i as f64)),
            (Int(i), Float(f)) => Float((i as f64) / f),
            (Float(f0), Float(f1)) => Float(f0 / f1),
        }
    }
}

impl<T> Rem<T> for Num
where
    Num: From<T>
{
    type Output = Self;

    fn rem(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => Float((i0 % i1) as f64),
            (Float(f), Int(i)) => Float(f % (i as f64)),
            (Int(i), Float(f)) => Float((i as f64) % f),
            (Float(f0), Float(f1)) => Float(f0 % f1),
        }
    }
}
