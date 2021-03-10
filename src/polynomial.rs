use core::ops::{Div, Shl, Sub};

/// Corresponding to the Finite Field Polynomials
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polynomial<const MAX_BYTES: usize>(pub [u8; MAX_BYTES]);

impl<const MAX_BYTES: usize> Polynomial<MAX_BYTES> {
    /// Shift all the bytes contained in the polynomial over by n bytes to the right
    fn rotate_left(&self, n: usize) -> Self {
        // Check that the n isn't bigger than the amount of bytes in the polynomial
        // If so, return a empty polynomial
        if n >= MAX_BYTES {
            return Polynomial([0; MAX_BYTES]);
        }

        let Polynomial(mut arr) = self;

        // Move all bytes over by n indices
        for i in 0..(MAX_BYTES - n) {
            arr[i] = arr[i + n];
        }

        // Set all leftover bytes to zero
        for i in (MAX_BYTES - n)..MAX_BYTES {
            arr[i] = 0x00;
        }

        Polynomial(arr)
    }

    /// Fetch the index of the most significant non-zero bit. Starting from 0 for the least
    /// significant bit.
    fn bit_len(&self) -> usize {
        let Polynomial(arr) = self;

        // Loop through all bytes (Most sign. to least sign.)
        for byte_index in 0..MAX_BYTES {
            let byte = arr[byte_index];

            // Loop through all bits (Most sign. to least sign.)
            for bit_index in 0..8 {
                if byte & (0x01 << (7 - bit_index)) != 0 {
                    return (MAX_BYTES * 8) - (byte_index * 8 + bit_index) - 1;
                }
            }
        }

        0
    }

    /// Return whether `self` is is 'greater' than `cmp`, meaning that `self` has the highest
    /// significant bit which `cmp` does not have.
    fn is_more_sign(&self, cmp: Self) -> bool {
        let Polynomial(self_arr) = self;
        let Polynomial(cmp_arr) = cmp;

        // Loop through all bytes (Most sign. to least sign.)
        for byte_index in 0..MAX_BYTES {
            let self_byte = self_arr[byte_index];
            let cmp_byte = cmp_arr[byte_index];

            // Shortcut for if both bytes are the same.
            if self_byte == cmp_byte {
                continue;
            }

            // Loop through all bits (Most sign. to least sign.)
            for bit_index in (0..8).rev() {
                let self_has_bit = self_byte & (0x01 << bit_index) != 0;
                let cmp_has_bit = cmp_byte & (0x01 << bit_index) != 0;

                // If both bits are the same continue to the next bit.
                if self_has_bit == cmp_has_bit {
                    continue;
                }

                // If the self then has a bit, we know that cmp does not have a bit.
                // Thus we know that self is more significant.
                // This also holds the other way round.
                return self_has_bit;
            }
        }

        false
    }

    /// Create a polynomial array with the poly byte at the first place.
    pub fn new_from_byte(byte: u8) -> Self {
        let mut arr = [0x00; MAX_BYTES];
        arr[MAX_BYTES - 1] = byte;
        Polynomial(arr)
    }
}

#[cfg(test)]
mod impls {
    use super::*;

    #[test]
    fn rotate_left() {
        macro_rules! rl_tv {
            ($bytes:expr, $amount:expr => $ans:expr) => {
                assert_eq!(Polynomial($bytes).rotate_left($amount), Polynomial($ans));
            };
        }

        rl_tv!([0, 1, 0, 2], 1 => [1, 0, 2, 0]);
        rl_tv!([0, 1, 0, 2], 3 => [2, 0, 0, 0]);
        rl_tv!([0, 1, 0, 2], 4 => [0, 0, 0, 0]);
    }

    #[test]
    fn bit_len() {
        macro_rules! bl_tv {
            ($bytes:expr => $ans:expr) => {
                assert_eq!(Polynomial($bytes).bit_len(), $ans);
            };
        }

        bl_tv!([0x00, 0b1000_0000] => 7);
        bl_tv!([0x00, 0b1000_0000, 0x00] => 7 + 8);
        bl_tv!([0x00, 0b0000_1000, 0x00] => 3 + 8);
        bl_tv!([0x00, 0b0000_0100, 0x00] => 2 + 8);
    }

    #[test]
    fn is_more_sign() {
        macro_rules! ms_tv {
            ($one:expr, $other:expr) => {
                assert!(Polynomial($one).is_more_sign(Polynomial($other)));
            };
            (> $one:expr, $other:expr) => {
                assert!(!Polynomial($one).is_more_sign(Polynomial($other)));
            };
        }

        ms_tv!([0x00, 0x08], [0x00, 0x07]);
        ms_tv!([0x80, 0x00], [0x00, 0x07]);
        ms_tv!(> [0x00, 0x07], [0x80, 0x00]);
    }
}

impl<const MAX_BYTES: usize> Shl<usize> for Polynomial<MAX_BYTES> {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        // TLDR: We first rotate all the bytes by the shift amount / 8.
        // Then we do actual shift actions on the two sequential bits.

        let Polynomial(mut rotated) = self.rotate_left(rhs / 8);
        let shl_amount = rhs % 8;

        // If the Shift Left Amount is now 0 we don't even have to bother.
        if shl_amount != 0 {
            // Go through all sequential byte pairs and shift them left.
            // We can do this since we now know that shl_amount < 8.
            for i in 0..(MAX_BYTES - 1) {
                rotated[i] = (rotated[i] << shl_amount) |        // ABCD EFGH => CDEF GH00
                             (rotated[i+1] >> (8 - shl_amount)); // ABCD EFGH => 0000 00AB
            }

            // Shift the last byte
            rotated[MAX_BYTES - 1] = rotated[MAX_BYTES - 1] << shl_amount;
        }

        Polynomial(rotated)
    }
}

#[test]
fn shift_left() {
    macro_rules! shl_tv {
        ($bytes:expr, $shl:expr => $ans:expr) => {
            assert_eq!(Polynomial($bytes) << $shl, Polynomial($ans));
        };
    }

    shl_tv!([0x00, 0xab], 8 => [0xab, 0x00]);
    shl_tv!([0x00, 0xab], 4 => [0x0a, 0xb0]);
    shl_tv!([0xcc, 0x33], 2 => [0x30, 0xcc]);
}

impl<const MAX_BYTES: usize> Sub for Polynomial<MAX_BYTES> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        // TLDR: Subtracting in the Finite Field just corresponds to XOR

        let mut result_arr = [0; MAX_BYTES];

        // Extract the arrays
        let Polynomial(x) = self;
        let Polynomial(y) = other;

        // Loop through all bytes, and XOR all of them
        for i in 0..MAX_BYTES {
            result_arr[i] = x[i] ^ y[i];
        }

        Self(result_arr)
    }
}

impl<const MAX_BYTES: usize> Div for Polynomial<MAX_BYTES> {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self {
        // TLDR: This function basically performs long division

        // As long as our numerator value is bigger than the denumerator, subtract a factor of
        // the denumerator from the numerator and repeat.
        while !rhs.is_more_sign(self) {
            // Shifting left by n bits basically means times 2^n and thus we are
            // subtracting a factor of the denumerator.
            //
            // Unless overflow happens here, but it shouldn't happen since self and rhs have the
            // same byte size.
            self = self - (rhs << (self.bit_len() - rhs.bit_len()));
        }

        self
    }
}

#[test]
fn div_test_vectors() {
    macro_rules! div_tv {
        ($num:expr, $denum:expr => $ans:expr) => {
            assert_eq!(Polynomial($num) / Polynomial($denum), Polynomial($ans));
        };
    }

    let num = [0x3f, 0x7e];
    let denum = [0x01, 0x1b];

    assert_eq!(Polynomial(num).bit_len(), 13);
    assert_eq!(Polynomial(denum).bit_len(), 8);

    assert_eq!(Polynomial(denum) << 5, Polynomial([0x23, 0x60]));

    div_tv!(num, denum => [0x01, 0x1a]);
}

#[cfg(test)]
impl<const MAX_BYTES: usize> core::fmt::Display for Polynomial<MAX_BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Polynomial: [")?;

        let Polynomial(arr) = self;
        for i in 0..MAX_BYTES {
            write!(f, "{:08b} ", arr[i])?;
        }

        write!(f, "]")
    }
}
