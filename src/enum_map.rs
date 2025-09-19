/// This file includes code from EnumMap (https://codeberg.org/xfix/enum-map)
/// which is licensed under the MIT License:
///
/// MIT License
///
/// Copyright (c) <year> <copyright holders>
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
use std::{
    ops::{Index, IndexMut},
    ptr, slice,
};

#[allow(dead_code)]
pub trait Enum: Sized {
    const COUNT: usize;

    /// Representation of an enum.
    ///
    /// For an enum with four elements it looks like this.
    ///
    /// ```
    /// type Array<V> = [V; 4];
    /// ```
    type Array<V>: Array;

    /// Takes an usize, and returns an element matching `into_usize` function.
    fn from_usize(value: usize) -> Self;
    /// Returns an unique identifier for a value within range of `0..Array::LENGTH`.
    fn into_usize(self) -> usize;

    fn all() -> impl Iterator<Item = Self> {
        (0..Self::Array::<()>::LENGTH).map(|i| Self::from_usize(i))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EnumMap<K: Enum, V> {
    array: K::Array<V>,
}

#[allow(dead_code)]
impl<K: Enum, V> EnumMap<K, V> {
    /// Creates an enum map from array.
    pub const fn from_array(array: K::Array<V>) -> EnumMap<K, V> {
        EnumMap { array }
    }

    pub fn as_slice(&self) -> &[V] {
        unsafe { slice::from_raw_parts(ptr::addr_of!(self.array).cast(), K::Array::<V>::LENGTH) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [V] {
        unsafe {
            slice::from_raw_parts_mut(ptr::addr_of_mut!(self.array).cast(), K::Array::<V>::LENGTH)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.as_slice()
            .iter()
            .enumerate()
            .map(|(i, v)| (K::from_usize(i), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.as_mut_slice()
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (K::from_usize(i), v))
    }
}

impl<K: Enum, V> Index<K> for EnumMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &V {
        &self.as_slice()[key.into_usize()]
    }
}

impl<K: Enum, V> IndexMut<K> for EnumMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut V {
        &mut self.as_mut_slice()[key.into_usize()]
    }
}

/// SAFETY: The array length needs to match actual storage.
pub unsafe trait Array {
    const LENGTH: usize;
}

unsafe impl<V, const N: usize> Array for [V; N] {
    const LENGTH: usize = N;
}

/// enum!(Type, length) requires that `Type` is a #[repr(u8)] simple enum, and `length` is the number of variants.
#[macro_export]
macro_rules! unsafe_simple_enum {
    {$name:ty, $n:literal} => {
        impl $crate::enum_map::Enum for $name {
            const COUNT: usize = $n;

            type Array<V> = [V; $n];

            fn from_usize(value: usize) -> Self {
                assert!(value < $n);
                unsafe { std::mem::transmute(value as u8) }
            }

            fn into_usize(self) -> usize {
                self as usize
            }
        }
    };
}

#[allow(unused_imports)]
pub use unsafe_simple_enum;
