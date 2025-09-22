use std::{
    array,
    ops::{Index, IndexMut},
};

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
