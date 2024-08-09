use core::{
    convert::{TryFrom, TryInto, From},
    ops::{Add, Div, Mul, Neg, Sub},
};

pub type Fixed16<const N: u64> = Fixed<N, i16>;

#[derive(Clone, Copy, Default, Ord, Eq, PartialEq, PartialOrd)]
pub struct Fixed<const N: u64, T>(T);

impl<const N: u64, T> From<T> for Fixed<N, T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<const N: u64, T: Copy> Fixed<N, T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn value(&self) -> T {
        self.0
    }
}

impl<const N: u64, T> Fixed<N, T>
where
    T: Copy + Into<i64> + TryFrom<i64> + TryFrom<u64> + Default,
{
    pub fn one() -> Self {
        Self(N.try_into().unwrap_or_default())
    }

    pub fn convert<const M: u64>(self) -> Fixed<M, T> {
        let value: i64 = (self.value().into() * M as i64) / N as i64;
        Fixed::new(value.try_into().unwrap_or_default())
    }

    pub fn to_frac(self, denom: T) -> Frac<T> {
        let v_i64 = (self.value().into() * denom.into()) / N as i64;
        Frac::new(v_i64.try_into().unwrap_or_default(), denom)
    }
}

impl<const N: u64, T: Copy + Add<Output = T>> Add for Fixed<N, T> {
    type Output = Fixed<N, T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.value() + rhs.value())
    }
}

impl<const N: u64, T: Copy + Sub<Output = T>> Sub for Fixed<N, T> {
    type Output = Fixed<N, T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.value() - rhs.value())
    }
}

impl<const N: u64, T> Mul for Fixed<N, T>
where
    T: Copy + Into<i64> + TryFrom<i64> + Default,
{
    type Output = Fixed<N, T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let v_i64: i64 = (self.value().into() * rhs.value().into()) / N as i64;
        Self::new(v_i64.try_into().unwrap_or_default())
    }
}

impl<const N: u64, T> Div for Fixed<N, T>
where
    T: Copy + Into<i64> + TryFrom<i64> + Default,
{
    type Output = Fixed<N, T>;

    fn div(self, rhs: Self) -> Self::Output {
        let v_i64: i64 = (self.value().into() * N as i64) / rhs.value().into();
        Self::new(v_i64.try_into().unwrap_or_default())
    }
}

impl<const N: u64, T: Copy + Neg<Output = T>> Neg for Fixed<N, T> {
    type Output = Fixed<N, T>;

    fn neg(self) -> Self::Output {
        Self::new(-self.value())
    }
}

impl<const N: u64, T> Mul<T> for Fixed<N, T>
where
    T: Copy + Mul<Output = T>
{
    type Output = Fixed<N, T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.value() * rhs)
    }
}

pub type Frac16 = Frac<i16>;
pub type FracU32 = Frac<u32>;

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

//impl<const MIN: i32, const MAX: i32> From<DFixed> for Fixed<MIN, MAX> {
//    fn from(value: DFixed) -> Self {
//        Self(value.convert(MIN, MAX).value())
//    }
//}
//
//impl<const MIN: i32, const MAX: i32> From<i32> for Fixed<MIN, MAX> {
//    fn from(value: i32) -> Self {
//        Fixed(i32::clamp(value, MIN, MAX))
//    }
//}
//
//impl<const MIN: i32, const MAX: i32> Fixed<MIN, MAX> {
//    pub fn new(value: i32, min: i32, max: i32) -> Self {
//        DFixed::new(value, min, max).into()
//    }
//
//    pub fn convert<const MIN2: i32, const MAX2: i32>(self) -> Fixed<MIN2, MAX2> {
//        let value = (self.value() - MIN) as u64;
//        let scaled = (value * (MAX2 - MIN2) as u64) / (MAX - MIN) as u64;
//        let value2 = (scaled as i64 + MIN2 as i64) as i32;
//
//        value2.into()
//    }
//
//    pub fn value(&self) -> i32 {
//        self.0
//    }
//}
//
//pub struct DFixed {
//    value: i32,
//    min: i32,
//    max: i32,
//}
//
//impl<const MIN: i32, const MAX: i32> From<Fixed<MIN, MAX>> for DFixed {
//    fn from(value: Fixed<MIN, MAX>) -> Self {
//        DFixed::new(value.value(), MIN, MAX)
//    }
//}
//
//impl DFixed {
//    pub fn new(value: i32, min: i32, max: i32) -> Self {
//        Self {
//            value: i32::clamp(value, min, max),
//            min,
//            max,
//        }
//    }
//
//    pub fn convert(self, min: i32, max: i32) -> DFixed {
//        let value = (self.value - self.min) as u64;
//        let scaled = (value * (max - min) as u64) / (self.max - self.min) as u64;
//        let value2 = (scaled as i64 + min as i64) as i32;
//
//        Self::new(value2, min, max)
//    }
//
//    pub fn value(&self) -> i32 {
//        self.value
//    }
//
//    pub fn min(&self) -> i32 {
//        self.min
//    }
//
//    pub fn max(&self) -> i32 {
//        self.max
//    }
//}
//

//pub fn add(left: u64, right: u64) -> u64 {
//    left + right
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn it_works() {
//        let result = add(2, 2);
//        assert_eq!(result, 4);
//    }
//}
