use encoding_rs::*;
use rbtree::RBTree;
use std::io::{BufReader, Read};
use std::fs::File;
use adler32::RollingAdler32;

use regex::Regex;
use byteorder::LittleEndian;

struct KeyInfo {
    tailerKeyText: Vec<u8>,
    headerKeyText: Vec<u8>,

    key_block_compressed_size_accumulator: u128,
    key_block_compressed_size: u128,
    key_block_decompressed_size: u128,
    num_entries: u128,
    num_entries_accumulator: u128,

}

struct Mdx {
    fname: String,
    //file path
    isCompact: bool,
    isStripKey: bool,
    isKeyCaseSensitive: bool,
    emptyStr: String,
    /**
     * encryption flag
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block
     */
    _encrypt: i16,
    _charset: String,
    _encoding: String,
    delimiter_width: i8,
    _passcode: String,
    _version: f32,
    _number_width: i32,
    _num_entries: u128,
    _num_key_blocks: u128,
    _num_record_blocks: u128,

    accumulation_blockId_tree: RBTree<i32, String>,
    _key_block_size: u128,
    _key_block_info_size: u128,
    _key_block_info_decomp_size: u128,
    _record_block_size: u128,
    _record_block_offset: u128,
    _record_block_start: u128,
    _key_block_offset: i32,
    _key_block_info_list: Vec<KeyInfo>,
    _record_info_struct_list: Vec<u8>,

    maxComRecSize: i32,
    maxDecompressedSize: u128,
    rec_decompressed_size: i32,

    maxDecomKeyBlockSize: u128,
    maxComKeyBlockSize: u128,

}

// 等价下面的代码
// from struct import unpack
// unpack unpack('>I', bytes_arr)
//  b"\x00\x00\x06V" => 1622
fn big_endian_bytes_unpack(byte: &[u8]) -> u32 {
    use byteorder::{BigEndian, ReadBytesExt};
    let mut buf_out: &[u8] = byte.clone();
    let num = buf_out.read_u32::<BigEndian>().unwrap();
    num
}

// unpack unpack('<I', bytes_arr)
// b"\x01*\xd2\x8b" => 2345806337
fn little_endian_bytes_unpack(byte: &[u8]) -> u32 {
    use byteorder::{LittleEndian, ReadBytesExt};
    let mut buf_out: &[u8] = byte.clone();
    let num = buf_out.read_u32::<LittleEndian>().unwrap();
    num
}


impl Mdx {
    fn new(f: String) -> Self {
        let mut reader = BufReader::new(File::open(&f).unwrap());
        let mut bytes1 = [0; 4];
        // read exactly 4 bytes
        reader.read_exact(&mut bytes1);
        let header_len = big_endian_bytes_unpack(&bytes1);
        let _key_block_offset: i32 = (4 + header_len + 4) as i32;
        let mut header_bytes = vec![0; header_len as usize];

        reader.read_exact(&mut header_bytes);

        // 4 bytes: adler32 checksum of header, in little endian
        let mut adler32_bytes = [0; 4];
        reader.read_exact(&mut adler32_bytes);
        let adler32 = little_endian_bytes_unpack(&adler32_bytes);


        let mut rolling_adler32 = RollingAdler32::new();
        rolling_adler32.update_buffer(&header_bytes);
        let header_hash = rolling_adler32.hash();
        println!("sum & 0xffffffff={},adler32={}", header_hash & 0xffffffff, adler32);
        if header_hash & 0xffffffff != adler32 as u32 {
            panic!("unrecognized format");
        }

        // header text in utf-16 encoding ending with '\x00\x00'
        let (header_left, end) = header_bytes.split_at(header_bytes.len() - 2);

        let re = Regex::new(r####"(\w+)=["](.*?)["]"####).unwrap();
        let hd_string = String::from_utf8_lossy(&header_bytes);
        println!("header string={}", hd_string);

        Mdx {
            fname: f.clone(),
            isCompact: false,
            isStripKey: false,
            isKeyCaseSensitive: false,
            emptyStr: "".to_string(),
            _encrypt: 0,
            _charset: "".to_string(),
            _encoding: "".to_string(),
            delimiter_width: 0,
            _passcode: "".to_string(),
            _version: 0.0,
            _number_width: 0,
            _num_entries: 0,
            _num_key_blocks: 0,
            _num_record_blocks: 0,
            accumulation_blockId_tree: RBTree::new(),
            _key_block_size: 0,
            _key_block_info_size: 0,
            _key_block_info_decomp_size: 0,
            _record_block_size: 0,
            _record_block_offset: 0,
            _record_block_start: 0,
            _key_block_offset,
            _key_block_info_list: vec![],
            _record_info_struct_list: vec![],
            maxComRecSize: 0,
            maxDecompressedSize: 0,
            rec_decompressed_size: 0,
            maxDecomKeyBlockSize: 0,
            maxComKeyBlockSize: 0,
        }
    }
}

fn main() {
    let mdx = Mdx::new(String::from("/home/cod3fn/code/rs-notes/resources/LSC4.mdx"));
    println!("mdx version {}", mdx._version);
    println!("mdx version {}", mdx.fname);
}