use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct SmallVec<T, const N: usize> {
    len: usize,
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> SmallVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        let len = self.len;
        self.len = 0;
        for item in &mut self.data[..len] {
            unsafe {
                item.assume_init_drop();
            }
        }
    }

    pub fn push(&mut self, value: T) {
        assert!(self.len < N);
        _ = self.data[self.len].write(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.data[self.len].assume_init_read() })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T, const N: usize> Default for SmallVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Deref for SmallVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { &*(&self.data[..self.len] as *const [MaybeUninit<T>] as *const [T]) }
    }
}

impl<T, const N: usize> DerefMut for SmallVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { &mut *(&mut self.data[..self.len] as *mut [MaybeUninit<T>] as *mut [T]) }
    }
}

impl<T, const N: usize> FromIterator<T> for SmallVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Self::new();
        for item in iter {
            vec.push(item);
        }
        vec
    }
}

impl<T, const N: usize> IntoIterator for SmallVec<T, N> {
    type Item = T;
    type IntoIter = SmallVecIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        SmallVecIter { v: self, index: 0 }
    }
}

#[derive(Debug)]
pub struct SmallVecIter<T, const N: usize> {
    v: SmallVec<T, N>,
    index: usize,
}

impl<T, const N: usize> Iterator for SmallVecIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < self.v.len {
            self.index += 1;
            let item = unsafe { self.v.data[index].assume_init_read() };
            Some(item)
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for SmallVecIter<T, N> {
    fn drop(&mut self) {
        let len = self.v.len;
        self.v.len = 0;
        for item in &mut self.v.data[self.index..len] {
            unsafe {
                item.assume_init_drop();
            }
        }
    }
}
