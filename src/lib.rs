//! A minimal heapless no_std implementation of 8-bit [cyclic redundancy
//! checks](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) in Rust. This
//! allows us to check for the integrity of data, and thus is mostly used when
//! transferring data over unstable or noisy connections. For example, this is connections with
//! embedded systems and network connections.
//!
//! Take a look at [the documentation](crate).
//!
//! # Features
//!
//! This crate provides the minimal functions needed to properly handle CRC's in an 8-bit
//! system. The provided functions are [`fetch_crc8`], [`has_valid_crc8`] and [`insert_crc8`]. This
//! should make handling most of the common CRC situations simple. Because of the minimalist
//! approach this crate takes, binary size should remain small. This especially fits well on
//! embedded hardware.
//!
//! # Usage
//!
//! Add this to your projects *Cargo.toml* with:
//!
//! ```toml
//! [dependencies]
//! crc8-rs = "1.1"
//! ```
//!
//! There are generally two ways to use this crate. We can use plain buffers or we wrap CRCs with
//! [`struct`](https://doc.rust-lang.org/std/keyword.struct.html) methods. Let us go over both
//! ways.
//!
//! ## Using plain buffers
//!
//! On the transferring end, we would similar code to the following.
//!
//! ```rust
//! use crc8_rs::{ has_valid_crc8, insert_crc8 };
//!
//! // We are given a data buffer we would like to transfer
//! // It is important to leave a unused byte at the end for the CRC byte
//! let data: [u8; 256] = [
//!     // ...snip
//! # 3; 256
//! ];
//!
//! // We can insert a CRC byte to the data buffer, this will be the last byte
//! // This time we use the generator polynomial of `0xD5`
//! let crc_data: [u8; 256] = insert_crc8(data, 0xD5);
//!
//! // Now we are able to verify that the CRC is valid
//! assert!(has_valid_crc8(crc_data, 0xD5));
//!
//! // Transfer the data...
//! ```
//!
//! Then on the receiving end, we would have code such as the following.
//!
//! ```rust
//! use crc8_rs::has_valid_crc8;
//!
//! // We receive the CRCed data from some source
//! // This buffer has the CRC byte as the last byte
//! let crc_data: [u8; 256] = // ...snip
//! # crc8_rs::insert_crc8([3; 256], 0xD5);
//!
//! // Now we can conditionally unpack it and use the data
//! if has_valid_crc8(crc_data, 0xD5) {
//!     // The data is contained in the crc_data
//!     let data = crc_data;
//!
//!     // ...snip
//! } else {
//!     panic!("CRC is invalid!")
//! }
//! ```
//!
//! ## Wrapping the CRC
//!
//! If we want to form packets from some given data, we may want to append a CRC byte when
//! transferring the data to verify the data's integrity.
//!
//! ```rust
//! use crc8_rs::insert_crc8;
//!
//! // Define a example packet structure
//! struct Packet {
//!     header:  [u8; 4],
//!     content: [u8; 247],
//!     footer:  [u8; 4],
//! }
//!
//! impl Packet {
//!     fn to_data_buffer(&self) -> [u8; 256] {
//!         let mut data = [0; 256];
//!         
//!         // First we insert the packet data into the buffer
//!         for i in 0..4   { data[i]       = self.header[i] }
//!         for i in 0..247 { data[i + 4]   = self.content[i] }
//!         for i in 0..4   { data[i + 251] = self.footer[i] }
//!
//!         // We use the generator polynomial `0xD5` here.
//!         insert_crc8(data, 0xD5)
//!     }
//! }
//! # // We add a little test here to make sure everything works.
//! # let pkt = Packet { header: [0xAB; 4], content: [0xCD; 247], footer: [0xEF; 4] };
//! # assert!(crc8_rs::has_valid_crc8(pkt.to_data_buffer(), 0xD5));
//! ```
//!
//! Receiving the given buffer is now quite simple.
//!
//! ```rust
//! use crc8_rs::has_valid_crc8;
//!
//! struct ReceivedPacket {
//!     header:  [u8; 4],
//!     content: [u8; 247],
//!     footer:  [u8; 4],
//! }
//!
//! impl ReceivedPacket {
//!     fn receive(data: [u8; 256]) -> Option<ReceivedPacket> {
//!         // Before we construct the instance, we first check the CRC
//!         if has_valid_crc8(data, 0xD6) {
//!             Some(ReceivedPacket {
//!                 // ...snip
//! #               header: {
//! #                   let mut header = [0; 4];
//! #                   for i in 0..4 {
//! #                       header[i] = data[i]
//! #                   }
//! #                   header
//! #               },
//! #               content: {
//! #                   let mut content = [0; 247];
//! #                   for i in 0..247 {
//! #                       content[i] = data[i + 4]
//! #                   }
//! #                   content
//! #               },
//! #               footer: {
//! #                   let mut footer = [0; 4];
//! #                   for i in 0..4 {
//! #                       footer[i] = data[i + 251]
//! #                   }
//! #                   footer
//! #               },
//!             })
//!         } else {
//!             None
//!         }
//!     }
//! }
//! # // We add a little test here to make sure everything works.
//! # assert!(ReceivedPacket::receive(crc8_rs::insert_crc8([0x42; 256], 0xD6)).is_some());
//! ```

#![warn(missing_docs)]
#![no_std]

//! The configuration used for a CRC-8 Process
struct Crc8Configuration {

}

mod polynomial;

use polynomial::Polynomial;

/// Determine whether a `data` buffer for a given generator `polynomial` has a valid CRC.
///
/// Will fetch the CRC value for the `data` buffer under the generator `polynomial` and return
/// whether it equals zero, which indicates the integrity of the data. It is a short hand for
/// [`fetch_crc8(data, polynomial) == 0`](crate::fetch_crc8).
///
/// # Examples
///
/// ```
/// use crc8_rs::{ has_valid_crc8, insert_crc8 };
///
/// const GENERATOR_POLYNOMIAL: u8 = 0xD5;
///
/// // We add an empty byte at the end for the CRC
/// let msg = b"Hello World!\0";
/// let msg = insert_crc8(*msg, GENERATOR_POLYNOMIAL);
///
/// // Will verify just fine!
/// assert!(has_valid_crc8(msg, GENERATOR_POLYNOMIAL));
///
/// let corrupted_msg = {
///     let mut tmp_msg = msg;
///     tmp_msg[1] = b'a';
///     tmp_msg
/// };
///
/// // The message is now corrupted and thus it can't verify the integrity!
/// assert!(!has_valid_crc8(corrupted_msg, GENERATOR_POLYNOMIAL));
/// ```
///
/// # Panics
///
/// The function will panic if given a zero-sized buffer. As can be seen in the following example.
///
/// ```should_panic
/// use crc8_rs::has_valid_crc8;
///
/// has_valid_crc8([], 0x42);
/// ```
pub fn has_valid_crc8<const DATA_SIZE: usize>(data: [u8; DATA_SIZE], polynomial: u8) -> bool {
    fetch_crc8(data, polynomial) == 0
}

/// Get the current CRC of a `data` buffer under a generator `polynomial`.
///
/// Calculates the polynomial modulo division of the `data` buffer with the `polynomial`. If we
/// give a valid CRC appended `data` buffer under `polynomial`, we will get `0` back. The
/// short-hand of this is the [`has_valid_crc8`] function. When given a null terminated `data`
/// buffer, the `fetch_crc8(data, polynomial) ^ polynomial` will equal the value needed to be set
/// as the last byte in order to get a valid CRC signed buffer. The short-hand of this is the
/// [`insert_crc8`] function.
///
/// # Examples
///
/// ```
/// use crc8_rs::{ insert_crc8, has_valid_crc8 };
///
/// // We can declare our packets ourselves
/// struct Packet {
///     header: u8,
///     content: [u8; 14],
/// }
///
/// impl Packet {
///     fn to_bytes(&self) -> [u8; 16] {
///         let mut data = [0; 16];
///
///         // Insert the packet data
///         data[0] = self.header;
///         for i in 0..14 { data[i + 1] = self.content[i] }
///
///         // Insert the CRC at the end of the buffer
///         // We use 0xD5 as the generator polynomial here
///         insert_crc8(data, 0xD5)
///     }
/// }
///
/// let pkt = Packet {
///     // ...
/// # header: b'H',
/// # content: *b"ello Everyone!",
/// };
/// assert!(has_valid_crc8(pkt.to_bytes(), 0xD5));
/// ```
///
/// # Panics
///
/// This function will panic when given a zero-sized buffer as can be seen in the following code
/// snippet.
///
/// ```should_panic
/// use crc8_rs::fetch_crc8;
///
/// fetch_crc8([], 0x42);
/// ```
pub fn fetch_crc8<const DATA_SIZE: usize>(data: [u8; DATA_SIZE], polynomial: u8) -> u8 {
    // Fetch the modulo division of the data with the generator polynomial
    let Polynomial(result_arr) = Polynomial(data) / Polynomial::new_from_byte(polynomial);

    // Then return the last byte
    result_arr[DATA_SIZE - 1]
}

/// Insert CRC byte in the last byte of `data` buffer under a generator `polynomial`.
///
/// This expects a last byte left for the CRC byte, any pre-existing last byte value will be
/// ignored and overwritten in the return value. This function is very similar to writing
/// [`data[data.len() - 1] = polynomial ^ fetch_crc8(data, polynomial)`](fetch_crc8).
///
/// # Examples
///
/// ```
/// use crc8_rs::{ has_valid_crc8, insert_crc8 };
///
/// const GENERATOR_POLYNOMIAL: u8 = 0xD5;
///
/// // We add an empty byte at the end for the CRC
/// let msg = b"Hello World!\0";
/// let msg = insert_crc8(*msg, GENERATOR_POLYNOMIAL);
///
/// // Will verify just fine!
/// assert!(has_valid_crc8(msg, GENERATOR_POLYNOMIAL));
///
/// let corrupted_msg = {
///     let mut tmp_msg = msg;
///     tmp_msg[1] = b'a';
///     tmp_msg
/// };
///
/// // The message is now corrupted and thus it can't verify the integrity!
/// assert!(!has_valid_crc8(corrupted_msg, GENERATOR_POLYNOMIAL));
/// ```
///
/// # Panics
///
/// This function will panic when given a zero-sized buffer as can be seen in the following code
/// snippet.
///
/// ```should_panic
/// use crc8_rs::insert_crc8;
///
/// insert_crc8([], 0x42);
/// ```
pub fn insert_crc8<const DATA_SIZE: usize>(
    mut data: [u8; DATA_SIZE],
    polynomial: u8,
) -> [u8; DATA_SIZE] {
    // Set the CRC byte to zero.
    data[DATA_SIZE - 1] = 0x00;

    // Fetch the crc and write to the last byte the byte which turns the crc into zero.
    data[DATA_SIZE - 1] = polynomial ^ fetch_crc8(data, polynomial);
    data
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

        assert!(has_valid_crc8(insert_crc8(test_vector, 0xA6), 0xA6));
    }
}
