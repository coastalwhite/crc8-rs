//! A no_std library for doing 8-bit cyclic redundancy checks. This is mostly meant
//! for embedded hardware, but it can be used in a std environment as well. This
//! uses const generics from __Rust 15.1__ which is available in stable from __Match
//! 25th, 2021__, before then you will have to use the __Rust__ beta.
//!
//! ## Usage
//!
//! ### Inserting and verifying corrupting byte arrays.
//!
//! ```
//! use crc8::{ verify_crc8, insert_crc8 };
//!
//! const GENERATOR_POLYNOMIAL: u8 = 0xD5;
//!
//! // We add an empty byte at the end for the CRC
//! let msg = b"Hello World!\0";
//! let msg = insert_crc8(&msg, GENERATOR_POLYNOMIAL);
//!
//! // Will verify just fine!
//! assert!(verify_crc8(&msg, GENERATOR_POLYNOMIAL));
//!
//! let corrupted_msg = {
//!     let mut tmp_msg = msg;
//!     tmp_msg[1] = b'a';
//!     tmp_msg
//! };
//!
//! // The message is now corrupted and thus it can't verify the integrity!
//! assert!(!verify_crc8(&corrupted_msg, GENERATOR_POLYNOMIAL));
//! ```
//!
//! ### Adding a CRC to custom Packet struct
//!
//! ```
//! use crc8::{ fetch_crc8, verify_crc8, concat_byte_arrays };
//!
//! const GENERATOR_POLYNOMIAL: u8 = 0xD5;
//!
//! // We can declare our packets ourselves
//! struct Packet {
//!     header: u8,
//!     content: [u8; 14],
//!     crc: u8,
//! }
//!
//! impl Packet {
//!     fn new(header: u8, content: [u8; 14]) -> Packet {
//!         let mut pkt = Packet {
//!             header,
//!             content,
//!             crc: 0,
//!         };
//!
//!         pkt.crc = GENERATOR_POLYNOMIAL ^ fetch_crc8(
//!             &pkt.to_bytes(),
//!             GENERATOR_POLYNOMIAL
//!         );
//!         pkt
//!     }
//!
//!     fn to_bytes(&self) -> [u8; 16] {
//!         concat_byte_arrays::<16, 15, 1>(
//!             concat_byte_arrays::<15, 1, 14>([self.header], self.content),
//!             [self.crc]
//!         )
//!     }
//! }
//!
//! assert!(verify_crc8(
//!     &Packet::new(b'H', *b"ello everyone!").to_bytes(),
//!     GENERATOR_POLYNOMIAL)
//! );
//! ```
//!
//!
//! ## License
//!
//! Licensed under a __MIT__ license.

#![warn(missing_docs)]
#![no_std]

mod polynomial;
use polynomial::Polynomial;

/// Verify the integrity of the bytes array.
///
/// Calculates the Cyclic Redundancy Check for the bytes array and return whether it equals
/// zero.
///
/// # Example
/// ```
/// use crc8::{ verify_crc8, insert_crc8 };
///
/// const GENERATOR_POLYNOMIAL: u8 = 0xD5;
///
/// // We add an empty byte at the end for the CRC
/// let msg = b"Hello World!\0";
/// let msg = insert_crc8(&msg, GENERATOR_POLYNOMIAL);
///
/// // Will verify just fine!
/// assert!(verify_crc8(&msg, GENERATOR_POLYNOMIAL));
///
/// let corrupted_msg = {
///     let mut tmp_msg = msg;
///     tmp_msg[1] = b'a';
///     tmp_msg
/// };
///
/// // The message is now corrupted and thus it can't verify the integrity!
/// assert!(!verify_crc8(&corrupted_msg, GENERATOR_POLYNOMIAL));
/// ```
pub fn verify_crc8<const BYTES: usize>(bytes: &[u8; BYTES], poly: u8) -> bool {
    fetch_crc8(bytes, poly) == 0
}

/// Generate Cyclic Redundancy Check for a given bytes array given a certain generator polynomial.
///
/// Calculates the Cyclic Redundancy Check for the bytes array and returns it.
///
/// # Example
/// ```
/// use crc8::{ fetch_crc8, verify_crc8, concat_byte_arrays };
///
/// const GENERATOR_POLYNOMIAL: u8 = 0xD5;
///
/// // We can declare our packets ourselves
/// struct Packet {
///     header: u8,
///     content: [u8; 14],
///     crc: u8,
/// }
///
/// impl Packet {
///     fn new(header: u8, content: [u8; 14]) -> Packet {
///         let mut pkt = Packet {
///             header,
///             content,
///             crc: 0,
///         };
///
///         pkt.crc = GENERATOR_POLYNOMIAL ^ fetch_crc8(
///             &pkt.to_bytes(),
///             GENERATOR_POLYNOMIAL
///         );
///         pkt
///     }
///
///     fn to_bytes(&self) -> [u8; 16] {
///         concat_byte_arrays::<16, 15, 1>(
///             concat_byte_arrays::<15, 1, 14>([self.header], self.content),
///             [self.crc]
///         )
///     }
/// }
///
/// assert!(verify_crc8(
///     &Packet::new(b'H', *b"ello everyone!").to_bytes(),
///     GENERATOR_POLYNOMIAL)
/// );
/// ```
pub fn fetch_crc8<const BYTES: usize>(bytes: &[u8; BYTES], poly: u8) -> u8 {
    let bytes = *bytes;

    let Polynomial(result_arr) = Polynomial(bytes) / Polynomial::new_from_byte(poly);
    let last_byte = result_arr[BYTES - 1];

    last_byte
}

/// Insert CRC on the last byte of a bytes array so that it can be verified.
///
/// This expects a last byte left for the CRC byte, any pre-existing last byte value will be
/// ignored overwritten in the return value.
///
/// # Example
/// ```
/// use crc8::{ verify_crc8, insert_crc8 };
///
/// const GENERATOR_POLYNOMIAL: u8 = 0xD5;
///
/// // We add an empty byte at the end for the CRC
/// let msg = b"Hello World!\0";
/// let msg = insert_crc8(&msg, GENERATOR_POLYNOMIAL);
///
/// // Will verify just fine!
/// assert!(verify_crc8(&msg, GENERATOR_POLYNOMIAL));
///
/// let corrupted_msg = {
///     let mut tmp_msg = msg;
///     tmp_msg[1] = b'a';
///     tmp_msg
/// };
///
/// // The message is now corrupted and thus it can't verify the integrity!
/// assert!(!verify_crc8(&corrupted_msg, GENERATOR_POLYNOMIAL));
/// ```
pub fn insert_crc8<const BYTES: usize>(bytes: &[u8; BYTES], poly: u8) -> [u8; BYTES] {
    let mut bytes = *bytes;

    // Set the CRC byte to zero.
    bytes[BYTES - 1] = 0x00;

    // Fetch the crc and write to the last byte the byte which turns the crc into zero.
    bytes[BYTES - 1] = poly ^ fetch_crc8(&bytes, poly);

    bytes
}

/// Concatenate two byte arrays into one byte array with compile time validation.
///
/// Safely concatenate two byte arrays of a given size which using the heap. The generics
/// arguments are the sizes of the result array, first array and second array,
/// respectively.
///
/// # Example
///
/// ```
/// use crc8::{ concat_byte_arrays };
///
/// let fst_msg = *b"Hi everyone! ";    // Length = 13
/// let snd_msg = *b"Pretty nifty ehh?"; // Length = 17
///
/// // Length will be 13 + 17 = 30
/// let msg = concat_byte_arrays::<30, 13, 17>(fst_msg, snd_msg);
///
/// assert_eq!(msg, *b"Hi everyone! Pretty nifty ehh?");
/// ```
pub fn concat_byte_arrays<const N: usize, const F: usize, const S: usize>(
    fst: [u8; F],
    snd: [u8; S],
) -> [u8; N] {
    // Do a compile time const generic bound check
    let _ = [0; 1][N - F - S];

    let mut result = [0; N];

    for i in 0..F {
        result[i] = fst[i];
    }

    for i in 0..S {
        result[F + i] = snd[i];
    }

    result
}

#[test]
fn crc_cycle() {
    let test_vectors = [
        [0x02, 0x30, 0xf0, 0x00],
        [0xff, 0x30, 0xf0, 0x00],
        [0x02, 0x56, 0xf0, 0x00],
        [0x02, 0x30, 0x49, 0x00],
        [0xab, 0xcd, 0xef, 0x00],
    ];

    for i in 0..test_vectors.len() {
        let test_vector = test_vectors[i];

        assert!(verify_crc8(&insert_crc8(&test_vector, 0xA6), 0xA6));
    }
}
