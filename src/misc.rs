use core::slice::{Iter, IterMut};

pub struct DSignal {
    value: i32,
    min: i32,
    max: i32,
}

impl<const MIN: i32, const MAX: i32> From<Signal<MIN, MAX>> for DSignal {
    fn from(value: Signal<MIN, MAX>) -> Self {
        DSignal::new(value.value(), MIN, MAX)
    }
}

impl DSignal {
    pub fn new(value: i32, min: i32, max: i32) -> Self {
        Self {
            value: i32::clamp(value, min, max),
            min,
            max,
        }
    }

    pub fn convert(self, min: i32, max: i32) -> DSignal {
        let value = (self.value - self.min) as u64;
        let scaled = (value * (max - min) as u64) / (self.max - self.min) as u64;
        let value2 = (scaled as i64 + min as i64) as i32;

        Self::new(value2, min, max)
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn min(&self) -> i32 {
        self.min
    }

    pub fn max(&self) -> i32 {
        self.max
    }
}

pub struct Signal<const MIN: i32, const MAX: i32>(i32);

impl<const MIN: i32, const MAX: i32> From<DSignal> for Signal<MIN, MAX> {
    fn from(value: DSignal) -> Self {
        Self(value.convert(MIN, MAX).value())
    }
}

impl<const MIN: i32, const MAX: i32> From<i32> for Signal<MIN, MAX> {
    fn from(value: i32) -> Self {
        Signal(i32::clamp(value, MIN, MAX))
    }
}

impl<const MIN: i32, const MAX: i32> Signal<MIN, MAX> {
    pub fn new(value: i32, min: i32, max: i32) -> Self {
        DSignal::new(value, min, max).into()
    }

    pub fn convert<const MIN2: i32, const MAX2: i32>(self) -> Signal<MIN2, MAX2> {
        let value = (self.value() - MIN) as u64;
        let scaled = (value * (MAX2 - MIN2) as u64) / (MAX - MIN) as u64;
        let value2 = (scaled as i64 + MIN2 as i64) as i32;

        value2.into()
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

pub struct FixedSet<T, const N: usize> {
    array: [T; N],
    n: usize,
}

impl<T: Eq + PartialEq + Copy + Clone + Default, const N: usize> FixedSet<T, N> {
    pub fn new() -> Self {
        Self {
            array: [T::default(); N],
            n: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.n
    }

    pub fn insert(&mut self, elem: T) -> bool {
        if self.n >= N || self.array.iter().any(|e| *e == elem) {
            return false;
        }
        self.array[self.n] = elem;
        self.n += 1;
        true
    }

    pub fn remove(&mut self, v: T) -> bool {
        let n_prev = self.n;

        self.n = 0;
        for i in 0..n_prev {
            if self.array[i] != v {
                self.array[self.n] = self.array[i];
                self.n += 1;
            }
        }

        self.n == n_prev
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.array[0..self.n].iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.array[0..self.n].iter_mut()
    }
}

// Helper functions
pub fn bitflags(flags: &[bool]) -> u8 {
    flags
        .into_iter()
        .enumerate()
        .fold(0, |b, (i, flag)| b | (*flag as u8) << i)
}

pub fn bitflag(flags: u8, i: u8) -> bool {
    (flags & (1 << i)) != 0
}

pub fn bits(byte: u8, start: u8, n_bits: u8) -> u8 {
    (byte << i32::max(0_i32, 8_i32 - start as i32 - n_bits as i32)) >> (8 - n_bits)
}
