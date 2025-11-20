#![cfg(all(target_arch = "x86_64", target_feature = "sse2"))]

use std::{
    arch::x86_64::{
        __m128i,
        // SSE2
        _mm_add_epi16,
        _mm_loadu_si128,
        _mm_storeu_si128,
        _mm_sub_epi16,
    },
    array,
    ops::{AddAssign, SubAssign},
};

pub struct Vector8<const N16: usize> {
    data: [__m128i; N16],
}

pub struct Vector16<const N8: usize> {
    data: [__m128i; N8],
}

pub struct Vector32<const N4: usize> {
    data: [__m128i; N4],
}

impl<const N: usize, const N16: usize> From<&[i8; N]> for Vector8<N16> {
    fn from(arr: &[i8; N]) -> Self {
        assert_eq!(N16 * 16, N);
        let data = array::from_fn(|i| unsafe {
            _mm_loadu_si128(arr.as_ptr().add(16 * i) as *const __m128i)
        });
        Self { data }
    }
}

impl<const N: usize, const N8: usize> From<&[i16; N]> for Vector16<N8> {
    fn from(arr: &[i16; N]) -> Self {
        assert_eq!(N8 * 8, N);
        let data = array::from_fn(|i| unsafe {
            _mm_loadu_si128(arr.as_ptr().add(8 * i) as *const __m128i)
        });
        Self { data }
    }
}

impl<const N: usize, const N4: usize> From<&[i32; N]> for Vector32<N4> {
    fn from(arr: &[i32; N]) -> Self {
        assert_eq!(N4 * 4, N);
        let data = array::from_fn(|i| unsafe {
            _mm_loadu_si128(arr.as_ptr().add(4 * i) as *const __m128i)
        });
        Self { data }
    }
}

impl<const N: usize, const N16: usize> From<&Vector8<N16>> for [i8; N] {
    fn from(vec: &Vector8<N16>) -> Self {
        assert_eq!(N16 * 16, N);
        let mut arr = [0i8; N];
        for (i, &x) in vec.data.iter().enumerate() {
            unsafe {
                _mm_storeu_si128(arr.as_mut_ptr().add(i * 16) as *mut __m128i, x);
            }
        }
        arr
    }
}

impl<const N: usize, const N8: usize> From<&Vector16<N8>> for [i16; N] {
    fn from(vec: &Vector16<N8>) -> Self {
        assert_eq!(N8 * 8, N);
        let mut arr = [0i16; N];
        for (i, &x) in vec.data.iter().enumerate() {
            unsafe {
                _mm_storeu_si128(arr.as_mut_ptr().add(i * 8) as *mut __m128i, x);
            }
        }
        arr
    }
}

impl<const N: usize, const N4: usize> From<&Vector32<N4>> for [i32; N] {
    fn from(vec: &Vector32<N4>) -> Self {
        assert_eq!(N4 * 4, N);
        let mut arr = [0i32; N];
        for (i, &x) in vec.data.iter().enumerate() {
            unsafe {
                _mm_storeu_si128(arr.as_mut_ptr().add(i * 4) as *mut __m128i, x);
            }
        }
        arr
    }
}

impl<const N8: usize> AddAssign<&Vector16<N8>> for Vector16<N8> {
    fn add_assign(&mut self, other: &Vector16<N8>) {
        for i in 0..N8 {
            self.data[i] = unsafe { _mm_add_epi16(self.data[i], other.data[i]) };
        }
    }
}

impl<const N8: usize> SubAssign<&Vector16<N8>> for Vector16<N8> {
    fn sub_assign(&mut self, other: &Vector16<N8>) {
        for i in 0..N8 {
            self.data[i] = unsafe { _mm_sub_epi16(self.data[i], other.data[i]) };
        }
    }
}

/// Multiply [M x N] * [N] + [M] -> [M]
/// 8 bit multiplications, 32 bit result
/// a * b + c
pub fn mul_add<const M: usize, const M4: usize, const N: usize, const N16: usize>(
    a: &[Vector8<N16>; M],
    b: &Vector8<N16>,
    c: &Vector32<M4>,
) -> Vector32<M4> {
    todo!()
}
