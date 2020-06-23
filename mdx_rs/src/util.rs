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