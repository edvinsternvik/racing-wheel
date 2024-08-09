use crate::fixed::Fixed;
use core::ops::{Div, Mul, Neg};

#[derive(Clone, Copy, Default)]
pub struct Frac<T>(T, T);

impl<T: Copy> Frac<T> {
    pub fn new(numerator: T, denomerator: T) -> Self {
        Self(numerator, denomerator)
    }

    pub fn value(&self) -> T {
        self.0
    }

    pub fn denom(&self) -> T {
        self.1
    }
}

impl<T> Frac<T>
where
    T: Copy + Into<i64> + TryFrom<i64> + TryFrom<u64> + Default,
{
    pub fn convert<const N: u64>(self) -> Fixed<N, T> {
        let v_i64: i64 = (self.value().into() * N as i64) / self.denom().into();
        Fixed::new(v_i64.try_into().unwrap_or_default())
    }
}

impl<const N: u64, T, U> Mul<Frac<U>> for Fixed<N, T>
where
    T: Copy + Into<i64> + TryFrom<i64> + Default,
    U: Copy + Into<i64> + TryFrom<i64> + Default,
{
    type Output = Fixed<N, T>;

    fn mul(self, rhs: Frac<U>) -> Self::Output {
        let v_i64: i64 = (self.value().into() * rhs.value().into()) / rhs.denom().into();
        Self::new(v_i64.try_into().unwrap_or_default())
    }
}

impl<const N: u64, T, U> Div<Frac<U>> for Fixed<N, T>
where
    T: Copy + Into<i64> + TryFrom<i64> + Default,
    U: Copy + Into<i64> + TryFrom<i64> + Default,
{
    type Output = Fixed<N, T>;

    fn div(self, rhs: Frac<U>) -> Self::Output {
        let v_i64: i64 = (self.value().into() * rhs.denom().into()) / rhs.value().into();
        Self::new(v_i64.try_into().unwrap_or_default())
    }
}

impl<T: Copy + Neg<Output = T>> Neg for Frac<T> {
    type Output = Frac<T>;

    fn neg(self) -> Self::Output {
        Self::new(-self.value(), self.denom())
    }
}
