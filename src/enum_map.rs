use std::{
    array,
    ops::{Index, IndexMut},
};

pub trait SimpleEnum: Sized {
    /// Representation of an enum.
    ///
    /// For an enum with four elements it looks like this.
    ///
    /// ```
    /// type Array<V> = [V; 4];
    /// ```
    type Array<V>: Array<Element = V>;

    /// Takes an usize, and returns an element matching `into_usize` function.
    fn from_usize(value: usize) -> Self;

    /// Returns an unique identifier for a value within range of `0..Array::LENGTH`.
    fn into_usize(self) -> usize;
}

/// SAFETY: enum!(Type, length) requires that `Type` is a #[repr(u8)] simple enum, and `length` is the number of variants.
#[macro_export]
macro_rules! unsafe_simple_enum {
    {$name:ty, $n:literal} => {
        impl $crate::enum_map::SimpleEnum for $name {
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

pub trait SimpleEnumExt: SimpleEnum {
    const COUNT: usize = Self::Array::<()>::LENGTH;

    fn all() -> impl Iterator<Item = Self> {
        (0..Self::COUNT).map(|i| Self::from_usize(i))
    }
}

impl<T: SimpleEnum> SimpleEnumExt for T {}

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
        let array = Array::from_fn(|i| f(K::from_usize(i)));
        EnumMap { array }
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.array
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, v)| (K::from_usize(i), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.array
            .as_mut_slice()
            .iter_mut()
            .enumerate()
            .map(|(i, v)| (K::from_usize(i), v))
    }
}

impl<K: SimpleEnum, V> Index<K> for EnumMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &V {
        &self.array[key.into_usize()]
    }
}

impl<K: SimpleEnum, V> IndexMut<K> for EnumMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut V {
        &mut self.array[key.into_usize()]
    }
}

pub trait Array: Index<usize, Output = Self::Element> + IndexMut<usize> {
    type Element;
    const LENGTH: usize;

    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> Self::Element;

    fn as_slice(&self) -> &[Self::Element];
    fn as_mut_slice(&mut self) -> &mut [Self::Element];
}

impl<V, const N: usize> Array for [V; N] {
    type Element = V;
    const LENGTH: usize = N;

    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> Self::Element,
    {
        array::from_fn(f)
    }

    fn as_slice(&self) -> &[Self::Element] {
        self.as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        self.as_mut_slice()
    }
}
