pub struct FixedSet<T, const N: usize> {
    array: [T; N],
    n: usize,
}

impl<T: Eq + Copy + Clone + Default, const N: usize> FixedSet<T, N> {
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
        if self.n >= N || self.items().iter().any(|e| *e == elem) {
            return false;
        }
        self.array[self.n] = elem;
        self.n += 1;
        true
    }

    pub fn remove(&mut self, v: T) -> bool {
        for (i, item) in self.items().iter().enumerate() {
            if *item == v {
                self.array[i] = self.array[self.n - 1];
                self.n -= 1;
                return true;
            }
        }
        false
    }

    pub fn items(&self) -> &[T] {
        &self.array[0..self.n]
    }
}

