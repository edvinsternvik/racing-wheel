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

    pub fn iter(&self) -> FixedSetIterator<'_, T, N> {
        FixedSetIterator {
            fixed_set: self,
            index: 0,
        }
    }
}

pub struct FixedSetIterator<'a, T, const N: usize> {
    fixed_set: &'a FixedSet<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for FixedSetIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.fixed_set.array.get(self.index - 1)
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
