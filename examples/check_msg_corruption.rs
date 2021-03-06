use crc8_rs::{ has_valid_crc8, insert_crc8 };

fn main() {
    let correct_msg = insert_crc8(*b"Hi!\0", 0xD5);

    assert!(has_valid_crc8(correct_msg, 0xD5));

    let mut incorrect_msg = correct_msg;
    incorrect_msg[1] = b'o';

    assert!(!has_valid_crc8(incorrect_msg, 0xD5));
}
