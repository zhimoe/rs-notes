use std::io::{BufReader, Read};
use std::fs::File;
use adler32::RollingAdler32;

use regex::Regex;
use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
use std::collections::HashMap;


// big endian bytes unpack,等价下面的代码
// from struct import unpack
// unpack('>I', bytes_arr)
//  b"\x00\x00\x06V" => 1622
pub fn be_bytes_unpack(byte: &[u8]) -> u32 {
    let mut out: &[u8] = byte.clone();
    let num = out.read_u32::<BigEndian>().unwrap();
    num
}

// unpack('<I', bytes_arr)
// b"\x01*\xd2\x8b" => 2345806337
pub fn le_bytes_unpack(byte: &[u8]) -> u32 {
    let mut out: &[u8] = byte.clone();
    let num = out.read_u32::<LittleEndian>().unwrap();
    num
}

// // python bytes.decode("utf-16").encode("utf-8")
// fn u8_to_utf16_string(bytes: &[u8]) -> Option<String> {
//     let (front, slice, back) = unsafe {
//         bytes.align_to::<u16>()
//     };
//     if front.is_empty() && back.is_empty() {
//         String::from_utf16(slice).ok()
//     } else {
//         None
//     }
// }
// 等价上面的u8_to_utf16_string方法
pub fn le_u8_string(slice: &[u8]) -> Option<String> {
    let idx = slice.len() / 2;
    let iter = (0..idx)
        .map(|i| u16::from_le_bytes([slice[2 * i], slice[2 * i + 1]]));

    std::char::decode_utf16(iter).collect::<Result<String, _>>().ok()
}

pub struct NumberBytes {
    bytes: Vec<u8>,
    tail: Vec<u8>,
}

impl NumberBytes {
    pub fn new(bytes: &Vec<u8>) -> Self {
        NumberBytes {
            bytes: bytes.clone(),
            tail: bytes.clone(),
        }
    }

    pub fn read_number(&mut self, width: usize) -> Option<u64> {
        let cur_tail = self.tail.clone();
        if cur_tail.len() < width {
            return None;
        }
        let (mut num, tail_bytes) = cur_tail.split_at(width);
        self.tail = Vec::from(tail_bytes);
        Some(num.read_u64::<BigEndian>().unwrap())
    }
}

pub fn adler32_checksum(contents: &Vec<u8>, adler32_bytes: &[u8], big_endian: bool) -> bool {
    let mut adler32: u32 = 0;
    if big_endian {
        adler32 = be_bytes_unpack(adler32_bytes);
    } else {
        adler32 = le_bytes_unpack(adler32_bytes);
    }
    let mut rolling_adler32 = RollingAdler32::new();
    rolling_adler32.update_buffer(contents);
    let hash = rolling_adler32.hash();
    if hash & 0xffffffff == adler32 {
        return true;
    }
    false
}
