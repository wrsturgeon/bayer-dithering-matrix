use core::mem;

#[inline]
const fn interleave_and_reverse_bits(a: usize, b: usize, highest_output_bit: u32) -> usize {
    let mut acc: usize = 0;
    let mut bit_o: usize = 1 << highest_output_bit;
    // NOTE that this uses at most the lower half of the bits in a `usize`:
    let mut bit_i: usize = 1;
    loop {
        if (a & bit_i) != 0 {
            acc |= bit_o;
        }
        bit_o = bit_o.unbounded_shr(1);

        if (b & bit_i) != 0 {
            acc |= bit_o;
        }
        bit_o = bit_o.unbounded_shr(1);

        if matches!(bit_o, 0) {
            return acc;
        }

        bit_i = bit_i.unbounded_shl(1);
    }
}

#[inline]
pub const fn compute_value_at_index(i: usize, j: usize, highest_output_bit: u32) -> usize {
    // https://en.wikipedia.org/wiki/Ordered_dithering#Threshold_map
    interleave_and_reverse_bits(i ^ j, i, highest_output_bit)
}

#[inline]
pub const fn matrix<T: Copy, const N: usize, const M: usize>() -> [[T; M]; N] {
    assert!(N > 0, "Cannot generate a zero-size Bayer dithering matrix.");
    assert!(M > 0, "Cannot generate a zero-size Bayer dithering matrix.");

    let mut uninit = [[mem::MaybeUninit::uninit(); M]; N];
    let mut ptr = uninit.as_mut_ptr() as *mut mem::MaybeUninit<T>;

    let index_bits_rounding_up = {
        let bits_rounding_up_n = ((N << 1) - 1).ilog2();
        let bits_rounding_up_m = ((M << 1) - 1).ilog2();
        let mut max = if bits_rounding_up_n > bits_rounding_up_m {
            bits_rounding_up_n
        } else {
            bits_rounding_up_m
        };
        max <<= 1;
        if let Some(sub) = max.checked_sub(1) {
            sub
        } else {
            0
        }
    };

    let just_past_max_representable = 1_usize.checked_shl((mem::size_of::<T>() << 3) as u32);

    let mut i = 0;
    loop {
        let mut j = 0;
        'j: loop {
            let entire_usize = compute_value_at_index(i, j, index_bits_rounding_up);

            #[cfg(debug_assertions)]
            if let Some(just_past_max_representable) = just_past_max_representable {
                assert!(
                    entire_usize < just_past_max_representable,
                    "It seems that the type you're using for Bayer matrix elements is too small to hold their values."
                );
            }

            let lower_bits = &entire_usize as *const usize as *const T;

            #[cfg(target_endian = "big")]
            {
                lower_bits = lower_bits
                    .byte_add(const { core::mem::size_of::<usize>() - core::mem::size_of::<T>() });
            }

            let _: &mut _ = unsafe { &mut *ptr }.write(unsafe { *lower_bits });
            if j == const { M - 1 } {
                if i == const { N - 1 } {
                    return unsafe {
                        *(&uninit as *const [[mem::MaybeUninit<T>; M]; N] as *const [[T; M]; N])
                    };
                }
                break 'j;
            }
            ptr = unsafe { ptr.add(1) };
            j += 1;
        }
        ptr = unsafe { ptr.add(1) };
        i += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1x1() {
        const MATRIX: [[u8; 1]; 1] = matrix();
        assert_eq!(MATRIX, [[0]]);
    }

    #[test]
    fn test_2x2() {
        const MATRIX: [[u8; 2]; 2] = matrix();
        assert_eq!(MATRIX, [[0, 2], [3, 1]]);
    }

    #[test]
    fn test_4x4() {
        const MATRIX: [[u8; 4]; 4] = matrix();
        assert_eq!(
            MATRIX,
            [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]],
        );
    }

    #[test]
    fn test_8x8() {
        const MATRIX: [[u8; 8]; 8] = matrix();
        assert_eq!(
            MATRIX,
            [
                [0, 32, 8, 40, 2, 34, 10, 42],
                [48, 16, 56, 24, 50, 18, 58, 26],
                [12, 44, 4, 36, 14, 46, 6, 38],
                [60, 28, 52, 20, 62, 30, 54, 22],
                [3, 35, 11, 43, 1, 33, 9, 41],
                [51, 19, 59, 27, 49, 17, 57, 25],
                [15, 47, 7, 39, 13, 45, 5, 37],
                [63, 31, 55, 23, 61, 29, 53, 21]
            ],
        );
    }

    /*
    #[test]
    fn test_16x16() {
        const MATRIX: [[u8; 16]; 16] = matrix();
        panic!("{MATRIX:?}");
    }
    */
}
