use std::array;

use extra::vector::{Vector8, Vector16, Vector32, mul_add};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[test]
fn test_vector8_conversion() {
    let arr: [i8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];
    let vec: Vector8<2> = (&arr).into();
    let arr_converted: [i8; 32] = (&vec).into();
    assert_eq!(arr_converted, arr);
}

#[test]
fn test_vector16_conversion() {
    let arr: [i16; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let vec: Vector16<2> = (&arr).into();
    let arr_converted: [i16; 16] = (&vec).into();
    assert_eq!(arr_converted, arr);
}

#[test]
fn test_vector32_conversion() {
    let arr: [i32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let vec: Vector32<2> = (&arr).into();
    let arr_converted: [i32; 8] = (&vec).into();
    assert_eq!(arr_converted, arr);
}

#[test]
fn test_vector16_add_assign() {
    let arr1: [i16; 16] = [1, 0x7f, 0xff, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let arr2: [i16; 16] = [0x7fff, 1, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let sum_expected: [i16; 16] = [
        -0x8000, 0x80, 0x100, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
    ];
    let vec1: Vector16<2> = (&arr1).into();
    let vec2: Vector16<2> = (&arr2).into();
    let mut vec_sum = vec1;
    vec_sum += &vec2;
    let sum: [i16; 16] = (&vec_sum).into();
    assert_eq!(sum, sum_expected);
}

#[test]
fn test_vector16_sub_assign() {
    let arr1: [i16; 16] = [
        -0x8000, 0x80, 0x100, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    let arr2: [i16; 16] = [1, 1, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let diff_expected: [i16; 16] = [0x7fff, 0x7f, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let vec1: Vector16<2> = (&arr1).into();
    let vec2: Vector16<2> = (&arr2).into();
    let mut vec_diff = vec1;
    vec_diff -= &vec2;
    let diff: [i16; 16] = (&vec_diff).into();
    assert_eq!(diff, diff_expected);
}

#[test]
fn test_mul_add() {
    let mut rng = StdRng::seed_from_u64(42);
    let a: [[i8; 32]; 8] = array::from_fn(|_| array::from_fn(|_| rng.random_range(-127..=127)));
    let b: [i8; 32] = array::from_fn(|_| rng.random_range(0..=127));
    let c: [i32; 8] = array::from_fn(|_| rng.random_range(-1_000_000..=1_000_000));
    const SHIFT: i32 = 1;

    let expected: [i32; 8] = array::from_fn(|y| {
        let mut sum: i32 = c[y];
        for x in 0..32 {
            sum += (a[y][x] as i32) * (b[x] as i32);
        }
        sum >> SHIFT
    });

    let a_vec: [Vector8<2>; 8] = a.map(|row| (&row).into());
    let b_vec: Vector8<2> = (&b).into();
    let c_vec: Vector32<2> = (&c).into();

    let result_vec = mul_add::<_, _, _, SHIFT>(&a_vec, &b_vec, &c_vec);
    let result: [i32; 8] = (&result_vec).into();

    assert_eq!(result, expected);
}
