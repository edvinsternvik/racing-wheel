use core::{
    convert::{TryFrom, TryInto, From},
    ops::{Add, Div, Mul, Neg, Sub},
};

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
