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

use self::Num::{Float, Int};
use super::super::SyntaxError;

type IntT = isize;

/// A numeric type that adapts its precision based on its usage.
#[derive(Clone, Copy, Debug, PartialOrd)]
pub enum Num {
    Float(f64),
    Int(IntT),
}

impl Num {
    #[must_use]
    pub fn abs(self) -> Self {
        match self {
            Float(f) => Float(f.abs()),
            Int(i) => {
                if let Some(i0) = i.checked_abs() {
                    Int(i0)
                } else {
                    Float((i as f64).abs())
                }
            }
        }
    }

    #[must_use]
    pub fn pow<T>(self, other: T) -> Self
    where
        Self: From<T>,
    {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => i0
                .checked_pow(i1 as u32)
                .map_or_else(|| Float((i0 as f64).powi(i1 as i32)), Int),
            (Float(f), Int(i)) => Float(f.powi(i as i32)),
            (Int(i), Float(f)) => Float((i as f64).powf(f)),
            (Float(f0), Float(f1)) => Float(f0.powf(f1)),
        }
    }

    #[must_use]
    pub fn is_nan(self) -> bool {
        if let Float(f) = self {
            f.is_nan()
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_infinite(self) -> bool {
        if let Float(f) = self {
            f.is_infinite()
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        if let Float(f) = self {
            f.is_finite()
        } else {
            true
        }
    }

    #[must_use]
    pub fn is_sign_positive(self) -> bool {
        match self {
            Float(f) => f.is_sign_positive(),
            Int(i) => i.is_positive(),
        }
    }

    #[must_use]
    pub fn is_sign_negative(self) -> bool {
        match self {
            Float(f) => f.is_sign_negative(),
            Int(i) => i.is_negative(),
        }
    }

    #[must_use]
    pub fn floor(self) -> Self {
        if let Float(f) = self {
            Int(f.floor() as IntT)
        } else {
            self
        }
    }

    #[must_use]
    pub fn ceil(self) -> Self {
        if let Float(f) = self {
            Int(f.ceil() as IntT)
        } else {
            self
        }
    }

    #[must_use]
    pub fn round(self) -> Self {
        if let Float(f) = self {
            Int(f.round() as IntT)
        } else {
            self
        }
    }

    #[must_use]
    pub fn trunc(self) -> Self {
        if let Float(f) = self {
            Int(f.trunc() as IntT)
        } else {
            self
        }
    }

    #[must_use]
    pub fn fract(self) -> Self {
        if let Float(f) = self {
            Float(f.fract())
        } else {
            Int(0)
        }
    }

    #[must_use]
    pub fn signum(self) -> Self {
        match self {
            Float(f) => Int(f.signum() as IntT),
            Int(i) => Int(i.signum()),
        }
    }

    #[must_use]
    pub fn recip(self) -> Self {
        Float(f64::from(self).recip())
    }

    #[must_use]
    pub fn sqrt(self) -> Self {
        Float(f64::from(self).sqrt())
    }

    #[must_use]
    pub fn cbrt(self) -> Self {
        Float(f64::from(self).cbrt())
    }

    #[must_use]
    pub fn exp(self) -> Self {
        Float(f64::from(self).exp())
    }

    #[must_use]
    pub fn ln(self) -> Self {
        Float(f64::from(self).ln())
    }

    #[must_use]
    pub fn exp2(self) -> Self {
        match self {
            Float(f) => Float(f.exp2()),
            Int(i) => Int((2 as IntT).pow(i as u32)),
        }
    }

    #[must_use]
    pub fn log2(self) -> Self {
        Float(f64::from(self).log2())
    }

    #[must_use]
    pub fn log10(self) -> Self {
        Float(f64::from(self).log10())
    }

    #[must_use]
    pub fn log<T>(self, other: T) -> Self
    where
        Self: From<T>,
    {
        Float(f64::from(self).log(f64::from(Self::from(other))))
    }

    #[must_use]
    pub fn hypot<T>(self, other: T) -> Self
    where
        Self: From<T>,
    {
        Float(f64::from(self).hypot(f64::from(Self::from(other))))
    }

    #[must_use]
    pub fn sin(self) -> Self {
        Float(f64::from(self).sin())
    }

    #[must_use]
    pub fn cos(self) -> Self {
        Float(f64::from(self).cos())
    }

    #[must_use]
    pub fn tan(self) -> Self {
        Float(f64::from(self).tan())
    }

    #[must_use]
    pub fn asin(self) -> Self {
        Float(f64::from(self).asin())
    }

    #[must_use]
    pub fn acos(self) -> Self {
        Float(f64::from(self).acos())
    }

    #[must_use]
    pub fn atan(self) -> Self {
        Float(f64::from(self).atan())
    }

    #[must_use]
    pub fn atan2<T>(self, other: T) -> Self
    where
        Self: From<T>,
    {
        Float(f64::from(self).atan2(f64::from(Self::from(other))))
    }

    #[must_use]
    pub fn to_degrees(self) -> Self {
        Float(f64::from(self).to_degrees())
    }

    #[must_use]
    pub fn to_radians(self) -> Self {
        Float(f64::from(self).to_radians())
    }
}

impl FromStr for Num {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<IntT>() {
            return Ok(Int(num));
        }

        if let Ok(num) = s.parse::<f64>() {
            return Ok(Float(num));
        }

        Err(SyntaxError::NotANumber(s.to_string()))
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
            (Float(f0), Float(f1)) => {
                f0 == INFINITY && f1 == INFINITY
                    || f0 == NEG_INFINITY && f1 == NEG_INFINITY
                    || (f0 - f1).abs() < EPSILON
            }
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
            Int(i) => match i.checked_neg() {
                Some(i0) => Int(i0),
                None => Float(-(i as f64)),
            },
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
            (Int(i0), Int(i1)) => i0
                .checked_add(i1)
                .map_or_else(|| Float((i0 as f64) + (i1 as f64)), Int),
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
            (Int(i0), Int(i1)) => i0
                .checked_sub(i1)
                .map_or_else(|| Float((i0 as f64) - (i1 as f64)), Int),
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
            (Int(i0), Int(i1)) => i0
                .checked_mul(i1)
                .map_or_else(|| Float((i0 as f64) * (i1 as f64)), Int),
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
            (Int(i0), Int(i1)) => {
                if let Some(0) = i0.checked_rem(i1) {
                    if let Some(i) = i0.checked_div(i1) {
                        return Int(i);
                    }
                }

                Float((i0 as f64) / (i1 as f64))
            }
            (Float(f), Int(i)) => Float(f / (i as f64)),
            (Int(i), Float(f)) => Float((i as f64) / f),
            (Float(f0), Float(f1)) => Float(f0 / f1),
        }
    }
}

impl<T> Rem<T> for Num
where
    Num: From<T>,
{
    type Output = Self;

    fn rem(self, other: T) -> Self::Output {
        match (self, other.into()) {
            (Int(i0), Int(i1)) => match i0.checked_rem(i1) {
                Some(i) => Int(i),
                None => Float((i0 as f64) % (i1 as f64)),
            },
            (Float(f), Int(i)) => Float(f % (i as f64)),
            (Int(i), Float(f)) => Float((i as f64) % f),
            (Float(f0), Float(f1)) => Float(f0 % f1),
        }
    }
}
