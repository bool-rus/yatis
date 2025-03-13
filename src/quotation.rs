use std::{iter::Sum, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};

use crate::t_types::{MoneyValue, Quotation};


const DIVIDER: i128 = 1_000_000_000;


fn from_quotation(q: Quotation) -> i128 {
    q.units as i128 * DIVIDER + q.nano as i128
}
fn to_quotation(n: i128) -> Quotation {
    Quotation { units: (n/DIVIDER) as i64, nano: (n % DIVIDER)  as i32}
}

impl Add for Quotation {
    type Output = Quotation;
    fn add(self, rhs: Self) -> Self::Output {
        let this = from_quotation(self);
        let rhs = from_quotation(rhs);
        to_quotation(this + rhs)
    }
}
impl Sub for Quotation {
    type Output = Quotation;
    fn sub(self, rhs: Self) -> Self::Output {
        let this = from_quotation(self);
        let rhs = from_quotation(rhs);
        to_quotation(this - rhs)
    }
}
impl<Rhs: Into<i128>> Mul<Rhs> for Quotation {
    type Output = Quotation;
    fn mul(self, rhs: Rhs) -> Self::Output {
        let this = from_quotation(self);
        to_quotation(this * rhs.into())
    }
}
impl Mul<Quotation> for Quotation {
    type Output = Quotation;
    fn mul(self, rhs: Quotation) -> Self::Output {
        let this = from_quotation(self);
        let rhs = from_quotation(rhs);
        to_quotation(this * rhs / DIVIDER)
    }
}
impl<Rhs: Into<i128>> Div<Rhs> for Quotation {
    type Output = Quotation;
    fn div(self, rhs: Rhs) -> Self::Output {
        let this = from_quotation(self);
        to_quotation(this / rhs.into())
    }
}
impl Div<Quotation> for Quotation {
    type Output = Quotation;
    fn div(self, rhs: Quotation) -> Self::Output {
        let this = from_quotation(self);
        let rhs = from_quotation(rhs);
        to_quotation(this * DIVIDER / rhs)
    }
}
impl From<MoneyValue> for Quotation {
    fn from(value: MoneyValue) -> Self {
        Self { units: value.units, nano: value.nano }
    }
}

// <some>Assign implementions
impl<T> MulAssign<T> for Quotation where Quotation: Mul<T>, <Quotation as Mul<T>>::Output: Into<Self> {
    fn mul_assign(&mut self, rhs: T) {
        *self = (*self * rhs).into()
    }
}
impl<T> DivAssign<T> for Quotation where Quotation: Div<T>, <Quotation as Div<T>>::Output: Into<Self> {
    fn div_assign(&mut self, rhs: T) {
        *self = (*self / rhs).into()
    }
}
impl<T> AddAssign<T> for Quotation where Quotation: Add<T>, <Quotation as Add<T>>::Output: Into<Self> {
    fn add_assign(&mut self, rhs: T) {
        *self = (*self + rhs).into()
    }
}
impl<T> SubAssign<T> for Quotation where Quotation: Sub<T>, <Quotation as Sub<T>>::Output: Into<Self> {
    fn sub_assign(&mut self, rhs: T) {
        *self = (*self - rhs).into()
    }
}
impl std::fmt::Display for Quotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fract: f32 = self.nano as f32 / DIVIDER as f32;
        write!(f, "{}", self.units as f32 + fract)
    }
}
impl Sum for Quotation {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        to_quotation(iter.map(|x|from_quotation(x)).sum())
    }
}

impl std::fmt::Display for MoneyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let q = Quotation {units: self.units, nano: self.nano};
        write!(f, "{} {}", q, self.currency)
    }
}

impl From<f64> for Quotation {
    fn from(value: f64) -> Self {
        Quotation { units: value.trunc() as i64, nano: (value.fract() * DIVIDER as f64) as i32 }
    }
}