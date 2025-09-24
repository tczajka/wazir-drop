use std::ops::{Index, IndexMut};

pub trait SimpleEnum: Sized {
    type Array<V>: Array<Element = V>;

    fn index(self) -> usize;
    fn from_index(value: usize) -> Self;
}

/// SAFETY: enum!(Type, length) requires that `Type` is a #[repr(u8)] simple enum, and `length` is the number of variants.
#[macro_export]
macro_rules! unsafe_simple_enum {
    {$name:ty, $n:literal} => {
        impl $crate::enum_map::SimpleEnum for $name {
            type Array<V> = [V; $n];

            fn index(self) -> usize {
                self as usize
            }

            fn from_index(value: usize) -> Self {
                assert!(value < $n);
                unsafe { std::mem::transmute(value as u8) }
            }

        }
    };
}

pub trait SimpleEnumExt: SimpleEnum {
    const COUNT: usize = Self::Array::<()>::LENGTH;

    fn all() -> impl Iterator<Item = Self> {
        (0..Self::COUNT).map(|i| Self::from_index(i))
    }
}

impl<T: SimpleEnum> SimpleEnumExt for T {}

use crate::array::Array;
pub use unsafe_simple_enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EnumMap<K: SimpleEnum, V> {
    array: K::Array<V>,
}

impl<K: SimpleEnum, V> EnumMap<K, V> {
    pub fn from_fn<F>(mut f: F) -> Self
    where
        F: FnMut(K) -> V,
    {
        let array = Array::from_fn(|i| f(K::from_index(i)));
        EnumMap { array }
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.array
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, v)| (K::from_index(i), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.array
            .as_mut_slice()
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (K::from_index(i), v))
    }
}

impl<K: SimpleEnum, V> Index<K> for EnumMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &V {
        &self.array[key.index()]
    }
}

impl<K: SimpleEnum, V> IndexMut<K> for EnumMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut V {
        &mut self.array[key.index()]
    }
}
