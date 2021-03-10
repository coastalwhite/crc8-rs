# CRC8

A no_std library for doing 8-bit cyclic redundancy checks. This is mostly meant
for embedded hardware, but it can be used in a std environment as well. This
uses const generics from __Rust 15.1__ which is available in stable from __Match
25th, 2021__, before then you will have to use the __Rust__ beta.

## Usage

This library provides 3 main functions.

### `fetch_crc8(bytes: &[u8; N], poly: u8) -> u8`

Returns the checksum for a given byte array and a given generator
polynomial, with the last bit being the Cyclic Redundancy Check.

[docs.rs](https://docs.rs/crc8/1.0.0/crc8/fn.fetch_crc8.html)

### `verify_crc8(bytes: &[u8; N], poly: u8) -> bool`

Verify that the given byte array with the given generator polynomial has a
checksum of zero.

[docs.rs](https://docs.rs/crc8/1.0.0/crc8/fn.verify_crc8.html)

### `insert_crc8(bytes: &[u8; N], poly: u8) -> [u8; N]`

Given a byte array (with the last byte left for the CRC) and a generator
polynomial, insert the CRC into the last byte of the byte array.

[docs.rs](https://docs.rs/crc8/1.0.0/crc8/fn.insert_crc8.html)

## License

Licensed under a __MIT license__.
