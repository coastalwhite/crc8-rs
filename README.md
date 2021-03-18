# crc8-rs

A minimal heapless no_std implementation of 8-bit [cyclic redundancy
checks](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) in Rust. This
allows us to check for the integrity of data, and thus is mostly used when
transferring data over unstable or noisy connections. For example, this is connections with
embedded systems and network connections.

Take a look at [the documentation](https://docs.rs/crc8-rs/).

## Features

This crate provides the minimal functions needed to properly handle CRC's in an 8-bit
system. The provided functions are
[`fetch_crc8`](https://docs.rs/crc8-rs/1.1.0/crc8_rs/fn.fetch_crc8.html),
[`has_valid_crc8`](https://docs.rs/crc8-rs/1.1.0/crc8_rs/fn.has_valid_crc8.html)
and [`insert_crc8`](https://docs.rs/crc8-rs/1.1.0/crc8_rs/fn.insert_crc8.html).
This
should make handling most of the common CRC situations simple. Because of the minimalist
approach this crate takes, binary size should remain small. This especially fits well on
embedded hardware.

## Usage

Add this to your projects *Cargo.toml* with:

```toml
[dependencies]
crc8-rs = "1.1"
```

There are generally two ways to use this crate. We can use plain buffers or we wrap CRCs with
[`struct`](https://doc.rust-lang.org/std/keyword.struct.html) methods. Let us go over both
ways.

### Using plain buffers

On the transferring end, we would similar code to the following.

```rust
use crc8_rs::{ has_valid_crc8, insert_crc8 };

// We are given a data buffer we would like to transfer
// It is important to leave a unused byte at the end for the CRC byte
let data: [u8; 256] = [
    // ...snip
];

// We can insert a CRC byte to the data buffer, this will be the last byte
// This time we use the generator polynomial of `0xD5`
let crc_data: [u8; 256] = insert_crc8(data, 0xD5);

// Now we are able to verify that the CRC is valid
assert!(has_valid_crc8(crc_data, 0xD5));

// Transfer the data...
```

Then on the receiving end, we would have code such as the following.

```rust
use crc8_rs::has_valid_crc8;

// We receive the CRCed data from some source
// This buffer has the CRC byte as the last byte
let crc_data: [u8; 256] = // ...snip

// Now we can conditionally unpack it and use the data
if has_valid_crc8(crc_data, 0xD5) {
    // The data is contained in the crc_data
    let data = crc_data;

    // ...snip
} else {
    panic!("CRC is invalid!")
}
```

### Wrapping the CRC

If we want to form packets from some given data, we may want to append a CRC byte when
transferring the data to verify the data's integrity.

```rust
use crc8_rs::insert_crc8;

// Define a example packet structure
struct Packet {
    header:  [u8; 4],
    content: [u8; 247],
    footer:  [u8; 4],
}

impl Packet {
    fn to_data_buffer(&self) -> [u8; 256] {
        let mut data = [0; 256];

        // First we insert the packet data into the buffer
        for i in 0..4   { data[i]       = self.header[i] }
        for i in 0..247 { data[i + 4]   = self.content[i] }
        for i in 0..4   { data[i + 251] = self.footer[i] }

        // We use the generator polynomial `0xD5` here.
        insert_crc8(data, 0xD5)
    }
}
```

Receiving the given buffer is now quite simple.

```rust
use crc8_rs::has_valid_crc8;

struct ReceivedPacket {
    header:  [u8; 4],
    content: [u8; 247],
    footer:  [u8; 4],
}

impl ReceivedPacket {
    fn receive(data: [u8; 256]) -> Option<ReceivedPacket> {
        // Before we construct the instance, we first check the CRC
        if has_valid_crc8(data, 0xD6) {
            Some(ReceivedPacket {
                // ...snip
            })
        } else {
            None
        }
    }
}
```

## License

Licensed with a MIT license.
