#![no_std]

use fixed::Fixed;
use fractional::Frac;

pub mod fixed;
pub mod fractional;

pub type Fixed16<const N: u64> = Fixed<N, i16>;
pub type FixedU16<const N: u64> = Fixed<N, u16>;

pub type Frac16 = Frac<i16>;
pub type FracU16 = Frac<u16>;
pub type FracU32 = Frac<u32>;

