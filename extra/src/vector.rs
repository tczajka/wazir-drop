#![cfg(all(
    target_arch = "x86_64",
    target_feature = "sse2",
    target_feature = "ssse3"
))]

use std::{
    array,
    ops::{AddAssign, SubAssign},
};

#[rustfmt::skip]
use std::arch::x86_64::{
    __m128i,
    // SSE2
    _mm_add_epi16,
    _mm_add_epi32,
    _mm_loadu_si128,
    _mm_madd_epi16,
    _mm_set1_epi16,
    _mm_setzero_si128,
    _mm_srai_epi32,
    _mm_storeu_si128,
    _mm_sub_epi16,
    // SSSE3
    _mm_hadd_epi32,
    _mm_maddubs_epi16,
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
        for (chunk, &m) in arr.chunks_exact_mut(16).zip(&vec.data) {
            unsafe {
                _mm_storeu_si128(chunk.as_ptr() as *mut __m128i, m);
            }
        }
        arr
    }
}

impl<const N: usize, const N8: usize> From<&Vector16<N8>> for [i16; N] {
    fn from(vec: &Vector16<N8>) -> Self {
        assert_eq!(N8 * 8, N);
        let mut arr = [0i16; N];
        for (chunk, &m) in arr.chunks_exact_mut(8).zip(&vec.data) {
            unsafe {
                _mm_storeu_si128(chunk.as_ptr() as *mut __m128i, m);
            }
        }
        arr
    }
}

impl<const N: usize, const N4: usize> From<&Vector32<N4>> for [i32; N] {
    fn from(vec: &Vector32<N4>) -> Self {
        assert_eq!(N4 * 4, N);
        let mut arr = [0i32; N];
        for (chunk, &m) in arr.chunks_exact_mut(4).zip(&vec.data) {
            unsafe {
                _mm_storeu_si128(chunk.as_ptr() as *mut __m128i, m);
            }
        }
        arr
    }
}

impl<const N8: usize> AddAssign<&Vector16<N8>> for Vector16<N8> {
    fn add_assign(&mut self, other: &Vector16<N8>) {
        for (a, &b) in self.data.iter_mut().zip(&other.data) {
            *a = unsafe { _mm_add_epi16(*a, b) };
        }
    }
}

impl<const N8: usize> SubAssign<&Vector16<N8>> for Vector16<N8> {
    fn sub_assign(&mut self, other: &Vector16<N8>) {
        for (a, &b) in self.data.iter_mut().zip(&other.data) {
            *a = unsafe { _mm_sub_epi16(*a, b) };
        }
    }
}

/// (a * b + c) >> SHIFT
/// [M x N] * [N] + [M] -> [M]
/// 8 bit multiplications, 32 bit result
/// a is signed -127..=127
/// b is unsigned 0..=127
pub fn mul_add<const M: usize, const M4: usize, const N16: usize, const SHIFT: i32>(
    a: &[Vector8<N16>; M],
    b: &Vector8<N16>,
    c: &Vector32<M4>,
) -> Vector32<M4> {
    assert_eq!(M4 * 4, M);

    let data = array::from_fn(|y4| {
        mul_add_4_rows::<_, SHIFT>(
            (&a[y4 * 4..(y4 + 1) * 4]).try_into().unwrap(),
            b,
            c.data[y4],
        )
    });
    Vector32 { data }
}

/// (a * b + c) >> SHIFT
/// [4 x N] * [N] + [4] -> [4]
/// 8 bit multiplications, 32 bit result
/// a is signed -127..=127
/// b is unsigned 0..=127
fn mul_add_4_rows<const N16: usize, const SHIFT: i32>(
    a: &[Vector8<N16>; 4],
    b: &Vector8<N16>,
    c: __m128i,
) -> __m128i {
    unsafe {
        // sums: [4 x 4]
        let mut sums = [_mm_setzero_si128(); 4];
        for x in 0..N16 {
            let bx = b.data[x];
            for y in 0..4 {
                let ax = a[y].data[x];
                // 16-bit dot products of 2
                let sum2 = _mm_maddubs_epi16(bx, ax);
                // 32-bit dot products of 4
                let sum4 = _mm_madd_epi16(sum2, _mm_set1_epi16(1));
                sums[y] = _mm_add_epi32(sums[y], sum4);
            }
        }
        // Now horizontally add each sums[y] and add c.
        // [0 0 1 1]
        let sums01 = _mm_hadd_epi32(sums[0], sums[1]);
        // [2 2 3 3]
        let sums23 = _mm_hadd_epi32(sums[2], sums[3]);
        // [0 1 2 3]
        let sums03 = _mm_hadd_epi32(sums01, sums23);
        let sum = _mm_add_epi32(sums03, c);
        _mm_srai_epi32(sum, SHIFT)
    }
}
